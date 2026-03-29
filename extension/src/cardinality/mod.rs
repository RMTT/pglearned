use pgrx::pg_guard;
use pgrx::pg_sys;
use pgrx::GucSetting;

use crate::rpc::client::PglRemoteSyncClient;
use crate::rpc::remote_server_url;

mod apply;
mod extract;
mod payload;

pub static PGL_ENABLE_REMOTE_CARDINALITY: GucSetting<bool> = GucSetting::<bool>::new(false);

static mut PREV_SET_REL_PATHLIST_HOOK: pg_sys::set_rel_pathlist_hook_type = None;
static mut PREV_SET_JOIN_PATHLIST_HOOK: pg_sys::set_join_pathlist_hook_type = None;

unsafe fn request_estimate(url: &str, payload: &payload::RelationEstimatePayload) -> Option<i64> {
    let request = match serde_json::to_string(payload) {
        Ok(request) => request,
        Err(err) => {
            pgrx::warning!("failed to serialize cardinality payload: {err}");
            return None;
        }
    };

    let mut client = match PglRemoteSyncClient::connect(url.to_string()) {
        Ok(client) => client,
        Err(err) => {
            pgrx::warning!("failed to connect to remote cardinality server: {err}");
            return None;
        }
    };

    match client.cardinality_estimate(vec![request]) {
        Ok(estimates) => estimates.into_iter().next(),
        Err(err) => {
            pgrx::warning!("failed to request cardinality estimate: {err}");
            None
        }
    }
}

#[pg_guard]
extern "C-unwind" fn pgl_set_rel_pathlist(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    rti: pg_sys::Index,
    rte: *mut pg_sys::RangeTblEntry,
) {
    unsafe {
        if !PGL_ENABLE_REMOTE_CARDINALITY.get() {
            if let Some(prev) = PREV_SET_REL_PATHLIST_HOOK {
                prev(root, rel, rti, rte);
            }
            return;
        }

        let remote_url = remote_server_url().unwrap_or_else(|| {
            pgrx::error!(
                "pgl.enable_remote_cardinality is on, but pgl.remote_server_url is not set"
            )
        });

        if let Some(prev) = PREV_SET_REL_PATHLIST_HOOK {
            prev(root, rel, rti, rte);
        }

        let estimate = extract::base_relation_payload(rel, rte)
            .as_ref()
            .and_then(|payload| request_estimate(&remote_url, payload));

        if let Some(estimate) = estimate {
            apply::apply_estimate_to_rel(rel, estimate);
        }
    }
}

#[pg_guard]
extern "C-unwind" fn pgl_set_join_pathlist(
    root: *mut pg_sys::PlannerInfo,
    joinrel: *mut pg_sys::RelOptInfo,
    outerrel: *mut pg_sys::RelOptInfo,
    innerrel: *mut pg_sys::RelOptInfo,
    jointype: pg_sys::JoinType::Type,
    extra: *mut pg_sys::JoinPathExtraData,
) {
    unsafe {
        if !PGL_ENABLE_REMOTE_CARDINALITY.get() {
            if let Some(prev) = PREV_SET_JOIN_PATHLIST_HOOK {
                prev(root, joinrel, outerrel, innerrel, jointype, extra);
            }
            return;
        }

        let remote_url = remote_server_url().unwrap_or_else(|| {
            pgrx::error!(
                "pgl.enable_remote_cardinality is on, but pgl.remote_server_url is not set"
            )
        });

        if let Some(prev) = PREV_SET_JOIN_PATHLIST_HOOK {
            prev(root, joinrel, outerrel, innerrel, jointype, extra);
        }

        let estimate = extract::join_relation_payload(root, joinrel, extra)
            .as_ref()
            .and_then(|payload| request_estimate(&remote_url, payload));

        if let Some(estimate) = estimate {
            apply::apply_estimate_to_rel(joinrel, estimate);
        }
    }
}

pub unsafe fn register() {
    PREV_SET_REL_PATHLIST_HOOK = pg_sys::set_rel_pathlist_hook;
    pg_sys::set_rel_pathlist_hook = Some(pgl_set_rel_pathlist);

    PREV_SET_JOIN_PATHLIST_HOOK = pg_sys::set_join_pathlist_hook;
    pg_sys::set_join_pathlist_hook = Some(pgl_set_join_pathlist);
}

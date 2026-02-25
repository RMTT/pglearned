use super::types::{PglPlannerMode, BRUTE_POSSIBLE_ARMS};
use super::{PGL_PLANNER_ARM, PGL_PLANNER_MODE, PGL_REMOTE_SERVER_URL};
use crate::utils::set_config_local;
use pgrx::pg_sys;
use std::ffi::CStr;

fn set_brute_planner_arm(arm: i32) -> anyhow::Result<()> {
    set_config_local("enable_hashjoin", &((arm & 1) != 0).to_string())?;
    set_config_local("enable_mergejoin", &((arm & 2) != 0).to_string())?;
    set_config_local("enable_nestloop", &((arm & 4) != 0).to_string())?;
    set_config_local("enable_indexscan", &((arm & 8) != 0).to_string())?;
    set_config_local("enable_seqscan", &((arm & 16) != 0).to_string())?;
    set_config_local("enable_indexonlyscan", &((arm & 32) != 0).to_string())?;
    Ok(())
}

unsafe fn planned_stmt_to_json(
    planned_stmt: *mut pg_sys::PlannedStmt,
    query_string: *const std::os::raw::c_char,
    bound_params: pg_sys::ParamListInfo,
) -> String {
    let es = pg_sys::NewExplainState();
    (*es).format = pg_sys::ExplainFormat::EXPLAIN_FORMAT_JSON;
    (*es).costs = true;
    (*es).buffers = true;
    (*es).timing = true;
    (*es).summary = true;

    pg_sys::ExplainBeginOutput(es);
    pg_sys::ExplainOnePlan(
        planned_stmt,
        std::ptr::null_mut(),
        es,
        query_string,
        bound_params,
        std::ptr::null_mut(),
        std::ptr::null(),
        std::ptr::null(),
        std::ptr::null(),
    );
    pg_sys::ExplainEndOutput(es);

    let str_info = (*es).str_;
    let c_str = CStr::from_ptr((*str_info).data);
    c_str.to_string_lossy().to_string()
}

pub unsafe fn pgl_brute_planner(
    parse: *mut pg_sys::Query,
    query_string: *const std::os::raw::c_char,
    cursor_options: i32,
    bound_params: pg_sys::ParamListInfo,
) -> *mut pg_sys::PlannedStmt {
    let mode = PGL_PLANNER_MODE.get();
    match mode {
        PglPlannerMode::Local => {
            let arm = PGL_PLANNER_ARM.get();
            if arm < 0 || arm > BRUTE_POSSIBLE_ARMS {
                pgrx::error!("wrong arm value, possible values: 0 -> {BRUTE_POSSIBLE_ARMS}");
            }

            if let Err(e) = set_brute_planner_arm(arm) {
                pgrx::error!("failed to set planner arm: {}", e);
            }

            pg_sys::standard_planner(parse, query_string, cursor_options, bound_params)
        }
        PglPlannerMode::Remote => {
            let mut plans = Vec::new();
            let mut candidate_stmts = Vec::new();

            for arm in 0..=BRUTE_POSSIBLE_ARMS {
                // We must copy the query tree because standard_planner modifies it
                let parse_copy = pg_sys::copyObjectImpl(parse as *const _) as *mut pg_sys::Query;

                if let Err(e) = set_brute_planner_arm(arm) {
                    pgrx::error!("failed to set planner arm: {}", e);
                }

                // Use the copy for planning
                let planned_stmt = pg_sys::standard_planner(
                    parse_copy,
                    query_string,
                    cursor_options,
                    bound_params,
                );

                candidate_stmts.push(planned_stmt);

                let mut json_str = planned_stmt_to_json(planned_stmt, query_string, bound_params);

                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    if let Some(arr) = json_val.as_array() {
                        if let Some(first) = arr.first() {
                            json_str = first.to_string();
                        }
                    }
                }

                plans.push(json_str);
            }

            let url = if let Some(url_cstr) = PGL_REMOTE_SERVER_URL.get() {
                url_cstr.to_string_lossy().to_string()
            } else {
                pgrx::error!("pgl.remote_server_url is not set");
            };

            let chosen_idx = match crate::rpc::client::PglRemoteSyncClient::connect(url) {
                Ok(mut client) => match client.choose_plan(plans) {
                    Ok(idx) => idx,
                    Err(e) => {
                        pgrx::error!("Failed to choose plan from remote: {}", e);
                    }
                },
                Err(e) => {
                    pgrx::error!("Failed to connect to remote planner: {}", e);
                }
            };

            if chosen_idx < 0 || chosen_idx as usize >= candidate_stmts.len() {
                pgrx::error!("Remote returned invalid arm index: {}", chosen_idx);
            }

            candidate_stmts[chosen_idx as usize]
        }
    }
}

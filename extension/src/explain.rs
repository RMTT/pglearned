use pgrx::pg_sys;
use pgrx::prelude::*;
use std::ffi::CString;

static mut PREV_EXPLAIN_PER_PLAN_HOOK: pg_sys::explain_per_plan_hook_type = None;
static mut PREV_EXPLAIN_PER_NODE_HOOK: pg_sys::explain_per_node_hook_type = None;

const PGL_SCHEMA_VERSION: i64 = 1;

#[pg_guard]
pub extern "C-unwind" fn pgl_explain_per_plan(
    plannedstmt: *mut pg_sys::PlannedStmt,
    into: *mut pg_sys::IntoClause,
    es: *mut pg_sys::ExplainState,
    query_string: *const std::os::raw::c_char,
    params: pg_sys::ParamListInfo,
    query_env: *mut pg_sys::QueryEnvironment,
) {
    unsafe {
        if let Some(prev) = PREV_EXPLAIN_PER_PLAN_HOOK {
            prev(plannedstmt, into, es, query_string, params, query_env);
        }

        let group_name = CString::new("PGL").unwrap();

        pg_sys::ExplainOpenGroup(group_name.as_ptr(), group_name.as_ptr(), true, es);

        let schema_version_label = CString::new("schema_version").unwrap();
        pg_sys::ExplainPropertyInteger(
            schema_version_label.as_ptr(),
            std::ptr::null(),
            PGL_SCHEMA_VERSION,
            es,
        );

        pg_sys::ExplainCloseGroup(group_name.as_ptr(), group_name.as_ptr(), true, es);
    }
}

#[pg_guard]
pub extern "C-unwind" fn pgl_explain_per_node(
    planstate: *mut pg_sys::PlanState,
    ancestors: *mut pg_sys::List,
    relationship: *const std::os::raw::c_char,
    plan_name: *const std::os::raw::c_char,
    es: *mut pg_sys::ExplainState,
) {
    unsafe {
        if let Some(prev) = PREV_EXPLAIN_PER_NODE_HOOK {
            prev(planstate, ancestors, relationship, plan_name, es);
        }
    }
}

pub unsafe fn register() {
    PREV_EXPLAIN_PER_PLAN_HOOK = pg_sys::explain_per_plan_hook;
    pg_sys::explain_per_plan_hook = Some(pgl_explain_per_plan);

    PREV_EXPLAIN_PER_NODE_HOOK = pg_sys::explain_per_node_hook;
    pg_sys::explain_per_node_hook = Some(pgl_explain_per_node);
}

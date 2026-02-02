mod brute;
mod default;
pub mod explain;
mod types;

use pgrx::pg_sys;
use pgrx::prelude::*;
use pgrx::GucSetting;
use std::ffi::CString;

use self::brute::pgl_brute_planner;
use self::default::pgl_default_planner;
use self::types::{PglPlannerMethod, PglPlannerMode};

pub use self::explain::EXPLAIN_PLANNER_MAP;

static mut PREV_PLANNER_HOOK: pg_sys::planner_hook_type = None;

pub static PGL_PLANNER_METHOD: GucSetting<PglPlannerMethod> =
    GucSetting::<PglPlannerMethod>::new(PglPlannerMethod::Default);
pub static PGL_PLANNER_ARM: GucSetting<i32> = GucSetting::<i32>::new(-1);
pub static PGL_REMOTE_SERVER_URL: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(None);
pub static PGL_PLANNER_MODE: GucSetting<PglPlannerMode> =
    GucSetting::<PglPlannerMode>::new(PglPlannerMode::Local);

#[pg_guard]
pub extern "C-unwind" fn pgl_planner(
    parse: *mut pg_sys::Query,
    query_string: *const std::os::raw::c_char,
    cursor_options: i32,
    bound_params: pg_sys::ParamListInfo,
) -> *mut pg_sys::PlannedStmt {
    unsafe {
        if let Some(prev) = PREV_PLANNER_HOOK {
            prev(parse, query_string, cursor_options, bound_params)
        } else {
            let method = PGL_PLANNER_METHOD.get();

            match method {
                PglPlannerMethod::Default => {
                    pgl_default_planner(parse, query_string, cursor_options, bound_params)
                }
                PglPlannerMethod::Brute => {
                    pgl_brute_planner(parse, query_string, cursor_options, bound_params)
                }
            }
        }
    }
}

pub unsafe fn register() {
    PREV_PLANNER_HOOK = pg_sys::planner_hook;
    pg_sys::planner_hook = Some(pgl_planner);
}

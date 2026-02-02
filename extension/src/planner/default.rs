use pgrx::pg_sys;

pub unsafe fn pgl_default_planner(
    parse: *mut pg_sys::Query,
    query_string: *const std::os::raw::c_char,
    cursor_options: i32,
    bound_params: pg_sys::ParamListInfo,
) -> *mut pg_sys::PlannedStmt {
    pg_sys::standard_planner(parse, query_string, cursor_options, bound_params)
}

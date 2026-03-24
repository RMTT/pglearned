use pgrx::pg_guard;
use pgrx::pg_sys;

static mut PREV_SET_REL_PATHLIST_HOOK: pg_sys::set_rel_pathlist_hook_type = None;
static mut PREV_SET_JOIN_PATHLIST_HOOK: pg_sys::set_join_pathlist_hook_type = None;

#[pg_guard]
extern "C-unwind" fn pgl_set_rel_pathlist(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    rti: pg_sys::Index,
    rte: *mut pg_sys::RangeTblEntry,
) {
    unsafe {
        if let Some(prev) = PREV_SET_REL_PATHLIST_HOOK {
            prev(root, rel, rti, rte);
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
        if let Some(prev) = PREV_SET_JOIN_PATHLIST_HOOK {
            prev(root, joinrel, outerrel, innerrel, jointype, extra);
        }
    }
}

pub unsafe fn register() {
    PREV_SET_REL_PATHLIST_HOOK = pg_sys::set_rel_pathlist_hook;
    pg_sys::set_rel_pathlist_hook = Some(pgl_set_rel_pathlist);

    PREV_SET_JOIN_PATHLIST_HOOK = pg_sys::set_join_pathlist_hook;
    pg_sys::set_join_pathlist_hook = Some(pgl_set_join_pathlist);
}

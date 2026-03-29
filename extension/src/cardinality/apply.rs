use pgrx::pg_sys;

fn clamp_row_estimate(estimate: i64) -> f64 {
    estimate.max(0) as f64
}

unsafe fn apply_estimate_to_path(path: *mut pg_sys::Path, estimate: f64) {
    if path.is_null() {
        return;
    }

    (*path).rows = estimate;

    if !(*path).param_info.is_null() {
        (*(*path).param_info).ppi_rows = estimate;
    }
}

unsafe fn apply_estimate_to_pathlist(pathlist: *mut pg_sys::List, estimate: f64) {
    if pathlist.is_null() {
        return;
    }

    let len = (*pathlist).length.max(0) as usize;
    let elements = (*pathlist).elements;

    for idx in 0..len {
        let path = (*elements.add(idx)).ptr_value as *mut pg_sys::Path;
        apply_estimate_to_path(path, estimate);
    }
}

pub unsafe fn apply_estimate_to_rel(rel: *mut pg_sys::RelOptInfo, estimate: i64) {
    if rel.is_null() {
        return;
    }

    let estimate = clamp_row_estimate(estimate);

    (*rel).rows = estimate;
    apply_estimate_to_pathlist((*rel).pathlist, estimate);
    apply_estimate_to_pathlist((*rel).partial_pathlist, estimate);
    apply_estimate_to_path((*rel).cheapest_startup_path, estimate);
    apply_estimate_to_path((*rel).cheapest_total_path, estimate);
    apply_estimate_to_path((*rel).cheapest_unique_path, estimate);
}

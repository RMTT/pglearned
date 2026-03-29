use std::ffi::CStr;

use pgrx::pg_sys;

use crate::utils::{bitmapset_members, cstr_to_string};

use super::payload::{EstimateKind, RelationEstimatePayload};

unsafe fn relation_name(rte: *mut pg_sys::RangeTblEntry) -> Option<String> {
    if rte.is_null() {
        return None;
    }

    let relname = pg_sys::get_rel_name((*rte).relid);
    if !relname.is_null() {
        let owned = CStr::from_ptr(relname).to_string_lossy().into_owned();
        pg_sys::pfree(relname.cast());
        return Some(owned);
    }

    cstr_to_string((*(*rte).eref).aliasname)
}

unsafe fn alias_name(rte: *mut pg_sys::RangeTblEntry) -> Option<String> {
    if rte.is_null() || (*rte).eref.is_null() {
        return None;
    }

    cstr_to_string((*(*rte).eref).aliasname)
}

unsafe fn relation_descriptors_from_relids(
    root: *mut pg_sys::PlannerInfo,
    relids: &[u32],
) -> (Vec<String>, Vec<String>) {
    let mut relation_names = Vec::new();
    let mut alias_names = Vec::new();

    if root.is_null() || (*root).simple_rte_array.is_null() {
        return (relation_names, alias_names);
    }

    for relid in relids {
        let rel_index = *relid as usize;
        if rel_index >= (*root).simple_rel_array_size as usize {
            continue;
        }

        let rte = *(*root).simple_rte_array.add(rel_index);
        if let Some(name) = relation_name(rte) {
            relation_names.push(name);
        }
        if let Some(alias) = alias_name(rte) {
            alias_names.push(alias);
        }
    }

    (relation_names, alias_names)
}

unsafe fn clause_strings(list: *mut pg_sys::List) -> Vec<String> {
    if list.is_null() {
        return Vec::new();
    }

    let len = (*list).length.max(0) as usize;
    let elements = (*list).elements;
    let mut clauses = Vec::with_capacity(len);

    for idx in 0..len {
        let restrict_info = (*elements.add(idx)).ptr_value as *mut pg_sys::RestrictInfo;
        if restrict_info.is_null() || (*restrict_info).clause.is_null() {
            continue;
        }

        let raw = pg_sys::nodeToString((*restrict_info).clause.cast());
        if raw.is_null() {
            continue;
        }

        clauses.push(CStr::from_ptr(raw).to_string_lossy().into_owned());
        pg_sys::pfree(raw.cast());
    }

    clauses
}

pub unsafe fn base_relation_payload(
    rel: *mut pg_sys::RelOptInfo,
    rte: *mut pg_sys::RangeTblEntry,
) -> Option<RelationEstimatePayload> {
    if rel.is_null() || rte.is_null() {
        return None;
    }

    let relation_names = relation_name(rte).into_iter().collect();
    let alias_names = alias_name(rte).into_iter().collect();
    let relids = bitmapset_members((*rel).relids);

    Some(RelationEstimatePayload {
        kind: EstimateKind::BaseRel,
        relids,
        relation_names,
        alias_names,
        clauses: clause_strings((*rel).baserestrictinfo),
        rows: (*rel).rows,
        tuples: ((*rel).tuples > 0.0).then_some((*rel).tuples),
    })
}

pub unsafe fn join_relation_payload(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    extra: *mut pg_sys::JoinPathExtraData,
) -> Option<RelationEstimatePayload> {
    if root.is_null() || rel.is_null() || extra.is_null() {
        return None;
    }

    let relids = bitmapset_members((*rel).relids);
    let (relation_names, alias_names) = relation_descriptors_from_relids(root, &relids);

    Some(RelationEstimatePayload {
        kind: EstimateKind::JoinRel,
        relids,
        relation_names,
        alias_names,
        clauses: clause_strings((*extra).restrictlist),
        rows: (*rel).rows,
        tuples: ((*rel).tuples > 0.0).then_some((*rel).tuples),
    })
}

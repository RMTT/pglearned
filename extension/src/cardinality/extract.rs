use std::ffi::CStr;
use std::ptr;

use pgrx::pg_sys;

use crate::utils::{bitmapset_members, cstr_to_string};

use super::payload::{
    EstimateKind, FilterPredicate, JoinPredicate, RelationEstimatePayload, RelationRef,
    TypedLiteral, CURRENT_PAYLOAD_VERSION,
};

const UNSUPPORTED_FILTER_SHAPE: &str = "unsupported_filter_shape";
const UNSUPPORTED_JOIN_SHAPE: &str = "unsupported_join_shape";
const UNSUPPORTED_JOIN_TYPE: &str = "unsupported_join_type";
const UNSUPPORTED_LITERAL_TYPE: &str = "unsupported_literal_type";
const UNSUPPORTED_WRAPPER: &str = "unsupported_wrapper";

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

unsafe fn relation_schema(rte: *mut pg_sys::RangeTblEntry) -> Option<String> {
    if rte.is_null() || (*rte).relid == pg_sys::InvalidOid {
        return None;
    }

    let namespace = pg_sys::get_rel_namespace((*rte).relid);
    if namespace == pg_sys::InvalidOid {
        return None;
    }

    let name = pg_sys::get_namespace_name(namespace);
    if name.is_null() {
        return None;
    }

    let owned = CStr::from_ptr(name).to_string_lossy().into_owned();
    pg_sys::pfree(name.cast());
    Some(owned)
}

unsafe fn current_database_name() -> Option<String> {
    if pg_sys::MyDatabaseId == pg_sys::InvalidOid {
        return None;
    }

    let name = pg_sys::get_database_name(pg_sys::MyDatabaseId);
    if name.is_null() {
        return None;
    }

    let owned = CStr::from_ptr(name).to_string_lossy().into_owned();
    pg_sys::pfree(name.cast());
    Some(owned)
}

unsafe fn relation_ref(rt_index: u32, rte: *mut pg_sys::RangeTblEntry) -> Option<RelationRef> {
    Some(RelationRef {
        rt_index,
        schema: relation_schema(rte),
        name: relation_name(rte)?,
        alias: alias_name(rte),
    })
}

#[derive(Debug, Clone, Default)]
struct VarDescriptor {
    relation: u32,
    schema: Option<String>,
    table_name: Option<String>,
    alias: Option<String>,
    column_name: Option<String>,
    attribute_number: Option<i16>,
}

unsafe fn attribute_name(relid: pg_sys::Oid, attnum: i16) -> Option<String> {
    if relid == pg_sys::InvalidOid || attnum <= 0 {
        return None;
    }

    let name = pg_sys::get_attname(relid, attnum, false);
    if name.is_null() {
        return None;
    }

    let owned = CStr::from_ptr(name).to_string_lossy().into_owned();
    pg_sys::pfree(name.cast());
    Some(owned)
}

unsafe fn rte_for_rt_index(
    root: *mut pg_sys::PlannerInfo,
    rt_index: u32,
    fallback_rte: *mut pg_sys::RangeTblEntry,
) -> *mut pg_sys::RangeTblEntry {
    if !root.is_null() && !(*root).simple_rte_array.is_null() {
        let rel_index = rt_index as usize;
        if rel_index < (*root).simple_rel_array_size as usize {
            let rte = *(*root).simple_rte_array.add(rel_index);
            if !rte.is_null() {
                return rte;
            }
        }
    }

    fallback_rte
}

unsafe fn describe_var(
    root: *mut pg_sys::PlannerInfo,
    var: *mut pg_sys::Var,
    fallback_rte: *mut pg_sys::RangeTblEntry,
) -> VarDescriptor {
    if var.is_null() {
        return VarDescriptor::default();
    }

    let relation = (*var).varno as u32;
    let attribute_number = ((*var).varattno > 0).then_some((*var).varattno);
    let rte = rte_for_rt_index(root, relation, fallback_rte);

    VarDescriptor {
        relation,
        schema: relation_schema(rte),
        table_name: relation_name(rte),
        alias: alias_name(rte),
        column_name: if rte.is_null() {
            None
        } else {
            attribute_name((*rte).relid, (*var).varattno)
        },
        attribute_number,
    }
}

unsafe fn relation_refs_from_relids(
    root: *mut pg_sys::PlannerInfo,
    relids: &[u32],
) -> Vec<RelationRef> {
    let mut relations = Vec::new();

    if root.is_null() || (*root).simple_rte_array.is_null() {
        return relations;
    }

    for relid in relids {
        let rel_index = *relid as usize;
        if rel_index >= (*root).simple_rel_array_size as usize {
            continue;
        }

        let rte = *(*root).simple_rte_array.add(rel_index);
        if let Some(relation) = relation_ref(*relid, rte) {
            relations.push(relation);
        }
    }

    relations
}

unsafe fn operator_name(opno: pg_sys::Oid) -> Option<String> {
    let name = pg_sys::get_opname(opno);
    if name.is_null() {
        return None;
    }

    let owned = CStr::from_ptr(name).to_string_lossy().into_owned();
    pg_sys::pfree(name.cast());
    Some(owned)
}

unsafe fn type_name(type_oid: pg_sys::Oid) -> Option<String> {
    let name = pg_sys::format_type_be(type_oid);
    if name.is_null() {
        return None;
    }

    let owned = CStr::from_ptr(name).to_string_lossy().into_owned();
    pg_sys::pfree(name.cast());
    Some(owned)
}

unsafe fn const_to_literal(value: *mut pg_sys::Const) -> Option<TypedLiteral> {
    if value.is_null() {
        return None;
    }

    let type_name = type_name((*value).consttype)?;
    if (*value).constisnull {
        return Some(TypedLiteral {
            type_name,
            type_oid: u32::from((*value).consttype),
            value: String::new(),
            is_null: true,
        });
    }

    let mut output_fn = pg_sys::InvalidOid;
    let mut is_varlena = false;
    pg_sys::getTypeOutputInfo((*value).consttype, &mut output_fn, &mut is_varlena);
    if output_fn == pg_sys::InvalidOid {
        return None;
    }

    let rendered = pg_sys::OidOutputFunctionCall(output_fn, (*value).constvalue);
    if rendered.is_null() {
        return None;
    }

    let rendered_value = CStr::from_ptr(rendered).to_string_lossy().into_owned();
    pg_sys::pfree(rendered.cast());

    Some(TypedLiteral {
        type_name,
        type_oid: u32::from((*value).consttype),
        value: rendered_value,
        is_null: false,
    })
}

unsafe fn unwrap_expr(mut expr: *mut pg_sys::Expr) -> Result<*mut pg_sys::Expr, &'static str> {
    loop {
        if expr.is_null() {
            return Err(UNSUPPORTED_FILTER_SHAPE);
        }

        match (*(expr as *mut pg_sys::Node)).type_ {
            pg_sys::NodeTag::T_RelabelType => {
                expr = (*(expr as *mut pg_sys::RelabelType)).arg;
            }
            pg_sys::NodeTag::T_Var | pg_sys::NodeTag::T_Const => return Ok(expr),
            pg_sys::NodeTag::T_FuncExpr | pg_sys::NodeTag::T_NullTest => {
                return Err(UNSUPPORTED_WRAPPER)
            }
            _ => return Err(UNSUPPORTED_FILTER_SHAPE),
        }
    }
}

unsafe fn binary_op_args(
    args: *mut pg_sys::List,
) -> Option<(*mut pg_sys::Expr, *mut pg_sys::Expr)> {
    if args.is_null() || (*args).length != 2 {
        return None;
    }

    let elements = (*args).elements;
    let left = (*elements).ptr_value as *mut pg_sys::Expr;
    let right = (*elements.add(1)).ptr_value as *mut pg_sys::Expr;
    Some((left, right))
}

unsafe fn clause_string(expr: *mut pg_sys::Expr) -> Option<String> {
    let raw = pg_sys::nodeToString(expr.cast());
    if raw.is_null() {
        return None;
    }

    let owned = CStr::from_ptr(raw).to_string_lossy().into_owned();
    pg_sys::pfree(raw.cast());
    Some(owned)
}

unsafe fn classify_clause(
    root: *mut pg_sys::PlannerInfo,
    clause: *mut pg_sys::Expr,
    fallback_rte: *mut pg_sys::RangeTblEntry,
) -> Result<(Option<FilterPredicate>, Option<JoinPredicate>), &'static str> {
    if clause.is_null() {
        return Err(UNSUPPORTED_FILTER_SHAPE);
    }

    if (*(clause as *mut pg_sys::Node)).type_ != pg_sys::NodeTag::T_OpExpr {
        return Err(UNSUPPORTED_FILTER_SHAPE);
    }

    let op_expr = clause as *mut pg_sys::OpExpr;
    let (left_raw, right_raw) = binary_op_args((*op_expr).args).ok_or(UNSUPPORTED_FILTER_SHAPE)?;
    let left = unwrap_expr(left_raw)?;
    let right = unwrap_expr(right_raw)?;
    let operator = operator_name((*op_expr).opno);
    let operator_oid =
        ((*op_expr).opno != pg_sys::InvalidOid).then_some(u32::from((*op_expr).opno));
    let clause = clause_string(clause).ok_or(UNSUPPORTED_FILTER_SHAPE)?;
    let left_tag = (*(left as *mut pg_sys::Node)).type_;
    let right_tag = (*(right as *mut pg_sys::Node)).type_;

    match (left_tag, right_tag) {
        (pg_sys::NodeTag::T_Var, pg_sys::NodeTag::T_Const) => {
            let left_var = describe_var(root, left as *mut pg_sys::Var, fallback_rte);
            Ok((
                Some(FilterPredicate {
                    clause,
                    left_relation: Some(left_var.relation),
                    schema: left_var.schema,
                    table_name: left_var.table_name,
                    alias: left_var.alias,
                    column_name: left_var.column_name,
                    attribute_number: left_var.attribute_number,
                    operator,
                    operator_oid,
                    right_literal: Some(
                        const_to_literal(right as *mut pg_sys::Const)
                            .ok_or(UNSUPPORTED_LITERAL_TYPE)?,
                    ),
                }),
                None,
            ))
        }
        (pg_sys::NodeTag::T_Const, pg_sys::NodeTag::T_Var) => {
            let left_var = describe_var(root, right as *mut pg_sys::Var, fallback_rte);
            Ok((
                Some(FilterPredicate {
                    clause,
                    left_relation: Some(left_var.relation),
                    schema: left_var.schema,
                    table_name: left_var.table_name,
                    alias: left_var.alias,
                    column_name: left_var.column_name,
                    attribute_number: left_var.attribute_number,
                    operator,
                    operator_oid,
                    right_literal: Some(
                        const_to_literal(left as *mut pg_sys::Const)
                            .ok_or(UNSUPPORTED_LITERAL_TYPE)?,
                    ),
                }),
                None,
            ))
        }
        (pg_sys::NodeTag::T_Var, pg_sys::NodeTag::T_Var) => {
            if operator.as_deref() != Some("=") {
                return Err(UNSUPPORTED_JOIN_SHAPE);
            }

            let left_var = describe_var(root, left as *mut pg_sys::Var, fallback_rte);
            let right_var = describe_var(root, right as *mut pg_sys::Var, fallback_rte);

            Ok((
                None,
                Some(JoinPredicate {
                    clause,
                    left_relation: left_var.relation,
                    right_relation: right_var.relation,
                    left_schema: left_var.schema,
                    left_table_name: left_var.table_name,
                    left_alias: left_var.alias,
                    left_column_name: left_var.column_name,
                    left_attribute_number: left_var.attribute_number,
                    right_schema: right_var.schema,
                    right_table_name: right_var.table_name,
                    right_alias: right_var.alias,
                    right_column_name: right_var.column_name,
                    right_attribute_number: right_var.attribute_number,
                    operator,
                    operator_oid,
                }),
            ))
        }
        (_, pg_sys::NodeTag::T_Const) | (pg_sys::NodeTag::T_Const, _) => {
            Err(UNSUPPORTED_LITERAL_TYPE)
        }
        _ => Err(UNSUPPORTED_FILTER_SHAPE),
    }
}

unsafe fn classify_restrictinfo_list(
    root: *mut pg_sys::PlannerInfo,
    list: *mut pg_sys::List,
    fallback_rte: *mut pg_sys::RangeTblEntry,
) -> (Vec<FilterPredicate>, Vec<JoinPredicate>, Vec<String>) {
    if list.is_null() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let len = (*list).length.max(0) as usize;
    let elements = (*list).elements;
    let mut filters = Vec::new();
    let mut joins = Vec::new();
    let mut unsupported_reasons = Vec::new();

    for idx in 0..len {
        let restrict_info = (*elements.add(idx)).ptr_value as *mut pg_sys::RestrictInfo;
        if restrict_info.is_null() || (*restrict_info).clause.is_null() {
            unsupported_reasons.push(UNSUPPORTED_FILTER_SHAPE.to_string());
            continue;
        }

        match classify_clause(root, (*restrict_info).clause, fallback_rte) {
            Ok((filter, join)) => {
                if let Some(filter) = filter {
                    filters.push(filter);
                }
                if let Some(join) = join {
                    joins.push(join);
                }
            }
            Err(reason) => unsupported_reasons.push(reason.to_string()),
        }
    }

    filters.sort_by(|left, right| {
        left.left_relation
            .cmp(&right.left_relation)
            .then_with(|| left.operator.cmp(&right.operator))
            .then_with(|| left.clause.cmp(&right.clause))
    });
    joins.sort_by(|left, right| {
        left.left_relation
            .cmp(&right.left_relation)
            .then_with(|| left.right_relation.cmp(&right.right_relation))
            .then_with(|| left.operator.cmp(&right.operator))
            .then_with(|| left.clause.cmp(&right.clause))
    });
    unsupported_reasons.sort();
    unsupported_reasons.dedup();

    (filters, joins, unsupported_reasons)
}

fn join_type_name(jointype: pg_sys::JoinType::Type) -> String {
    match jointype {
        pg_sys::JoinType::JOIN_INNER => "inner",
        pg_sys::JoinType::JOIN_LEFT => "left",
        pg_sys::JoinType::JOIN_FULL => "full",
        pg_sys::JoinType::JOIN_RIGHT => "right",
        pg_sys::JoinType::JOIN_SEMI => "semi",
        pg_sys::JoinType::JOIN_ANTI => "anti",
        pg_sys::JoinType::JOIN_RIGHT_SEMI => "right_semi",
        pg_sys::JoinType::JOIN_RIGHT_ANTI => "right_anti",
        _ => "unknown",
    }
    .to_string()
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
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    rte: *mut pg_sys::RangeTblEntry,
) -> Option<RelationEstimatePayload> {
    if rel.is_null() || rte.is_null() {
        return None;
    }

    let relation_names = relation_name(rte).into_iter().collect();
    let alias_names = alias_name(rte).into_iter().collect();
    let relids = bitmapset_members((*rel).relids);
    let rt_index = relids.first().copied().unwrap_or_default();
    let (filters, joins, unsupported_reasons) =
        classify_restrictinfo_list(root, (*rel).baserestrictinfo, rte);

    Some(RelationEstimatePayload {
        payload_version: CURRENT_PAYLOAD_VERSION,
        kind: EstimateKind::BaseRel,
        join_type: None,
        database: current_database_name(),
        db_oid: (pg_sys::MyDatabaseId != pg_sys::InvalidOid)
            .then_some(u32::from(pg_sys::MyDatabaseId)),
        state_key: None,
        rt_indexes: relids.clone(),
        relids,
        relation_names,
        alias_names,
        clauses: clause_strings((*rel).baserestrictinfo),
        relations: relation_ref(rt_index, rte).into_iter().collect(),
        filters,
        joins,
        fully_supported: unsupported_reasons.is_empty(),
        unsupported_reasons,
        rows: (*rel).rows,
        tuples: ((*rel).tuples > 0.0).then_some((*rel).tuples),
    })
}

pub unsafe fn join_relation_payload(
    root: *mut pg_sys::PlannerInfo,
    rel: *mut pg_sys::RelOptInfo,
    jointype: pg_sys::JoinType::Type,
    extra: *mut pg_sys::JoinPathExtraData,
) -> Option<RelationEstimatePayload> {
    if root.is_null() || rel.is_null() || extra.is_null() {
        return None;
    }

    let relids = bitmapset_members((*rel).relids);
    let (relation_names, alias_names) = relation_descriptors_from_relids(root, &relids);
    let relations = relation_refs_from_relids(root, &relids);
    let (filters, joins, mut unsupported_reasons) =
        classify_restrictinfo_list(root, (*extra).restrictlist, ptr::null_mut());
    let join_type = join_type_name(jointype);
    if jointype != pg_sys::JoinType::JOIN_INNER {
        unsupported_reasons.push(UNSUPPORTED_JOIN_TYPE.to_string());
    }

    Some(RelationEstimatePayload {
        payload_version: CURRENT_PAYLOAD_VERSION,
        kind: EstimateKind::JoinRel,
        join_type: Some(join_type),
        database: current_database_name(),
        db_oid: (pg_sys::MyDatabaseId != pg_sys::InvalidOid)
            .then_some(u32::from(pg_sys::MyDatabaseId)),
        state_key: None,
        rt_indexes: relids.clone(),
        relids,
        relation_names,
        alias_names,
        clauses: clause_strings((*extra).restrictlist),
        relations,
        filters,
        joins,
        fully_supported: unsupported_reasons.is_empty(),
        unsupported_reasons,
        rows: (*rel).rows,
        tuples: ((*rel).tuples > 0.0).then_some((*rel).tuples),
    })
}

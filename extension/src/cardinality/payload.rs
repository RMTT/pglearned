use serde::Serialize;

pub const CURRENT_PAYLOAD_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EstimateKind {
    BaseRel,
    JoinRel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelationRef {
    pub rt_index: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TypedLiteral {
    pub type_name: String,
    pub type_oid: u32,
    pub value: String,
    pub is_null: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FilterPredicate {
    pub clause: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_relation: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribute_number: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_oid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_literal: Option<TypedLiteral>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JoinPredicate {
    pub clause: String,
    pub left_relation: u32,
    pub right_relation: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_table_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_column_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_attribute_number: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_table_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_column_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_attribute_number: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_oid: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RelationEstimatePayload {
    pub payload_version: u32,
    pub kind: EstimateKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub join_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_oid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_key: Option<String>,
    pub relids: Vec<u32>,
    pub relation_names: Vec<String>,
    pub alias_names: Vec<String>,
    pub clauses: Vec<String>,
    pub rt_indexes: Vec<u32>,
    pub relations: Vec<RelationRef>,
    pub filters: Vec<FilterPredicate>,
    pub joins: Vec<JoinPredicate>,
    pub fully_supported: bool,
    pub unsupported_reasons: Vec<String>,
    pub rows: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tuples: Option<f64>,
}

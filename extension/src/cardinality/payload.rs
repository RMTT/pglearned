use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EstimateKind {
    BaseRel,
    JoinRel,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RelationEstimatePayload {
    pub kind: EstimateKind,
    pub relids: Vec<u32>,
    pub relation_names: Vec<String>,
    pub alias_names: Vec<String>,
    pub clauses: Vec<String>,
    pub rows: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tuples: Option<f64>,
}

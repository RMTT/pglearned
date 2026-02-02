pub const BRUTE_POSSIBLE_ARMS: i32 = (1 << 6) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, pgrx::PostgresGucEnum)]
pub enum PglPlannerMode {
    Local,
    Remote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, pgrx::PostgresGucEnum)]
pub enum PglPlannerMethod {
    Default,
    Brute,
}

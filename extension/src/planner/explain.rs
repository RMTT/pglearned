use super::types::BRUTE_POSSIBLE_ARMS;
use crate::utils::set_config_local;
use std::any::Any;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct ExplainConfiguerState {
    storage: HashMap<String, Box<dyn Any>>,
}

#[allow(dead_code)]
impl ExplainConfiguerState {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn insert<T: 'static>(&mut self, key: impl Into<String>, value: T) {
        self.storage.insert(key.into(), Box::new(value));
    }

    pub fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.storage.get(key)?.downcast_ref::<T>()
    }
}

type IterExplain = fn(&str, i32) -> Box<dyn Iterator<Item = String>>;
type SetupExplain = fn() -> anyhow::Result<ExplainConfiguerState>;
type CleanupExplain = fn(&ExplainConfiguerState) -> anyhow::Result<()>;
pub struct ExplainConfiguer {
    pub make_iter: IterExplain,
    pub setup: SetupExplain,
    pub cleanup: CleanupExplain,
}

pub static EXPLAIN_PLANNER_MAP: phf::Map<&'static str, ExplainConfiguer> = phf::phf_map! {
    "default" => ExplainConfiguer {
        make_iter: default_explain_iterator,
        setup: default_explain_setup,
        cleanup: default_explain_cleanup
    },
    "brute" => ExplainConfiguer {
        make_iter: brute_explain_iterator,
        setup: brute_explain_setup,
        cleanup: brute_explain_cleanup
    }
};

fn default_explain_setup() -> anyhow::Result<ExplainConfiguerState> {
    let state = ExplainConfiguerState::new();
    set_config_local("pgl.planner_method", "default")?;
    Ok(state)
}
fn default_explain_cleanup(_: &ExplainConfiguerState) -> anyhow::Result<()> {
    Ok(())
}
fn default_explain_iterator(query: &str, _: i32) -> Box<dyn Iterator<Item = String>> {
    let default = vec![query.to_string()];
    Box::new(default.into_iter())
}

fn brute_explain_setup() -> anyhow::Result<ExplainConfiguerState> {
    let state = ExplainConfiguerState::new();
    set_config_local("pgl.planner_method", "brute")?;
    Ok(state)
}
fn brute_explain_cleanup(_: &ExplainConfiguerState) -> anyhow::Result<()> {
    Ok(())
}
fn brute_explain_iterator(query: &str, arm: i32) -> Box<dyn Iterator<Item = String>> {
    if arm < -1 || arm > BRUTE_POSSIBLE_ARMS {
        pgrx::error!("wrong arm value, possible values: -1 -> {BRUTE_POSSIBLE_ARMS}");
    }

    let mut arm_start: i32 = arm;
    let mut arm_end: i32 = arm;
    if arm == -1 {
        arm_start = 0;
        arm_end = BRUTE_POSSIBLE_ARMS;
    }
    let query_string = query.to_string();
    let mut current_arm = arm_start;
    Box::new(std::iter::from_fn(move || {
        if current_arm > arm_end {
            None
        } else {
            set_config_local("pgl.planner_arm", &current_arm.to_string()).unwrap();

            current_arm += 1;
            Some(query_string.clone())
        }
    }))
}

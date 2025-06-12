//! Top-level `WorkflowDSL` definition **plus helper methods** used by the engine.
//!
//! This file stays in the *DSL crate* (stepflow-dsl) so that the runtime engine can
//! call strongly-typed helpers without re-implementing reflection logic.
//!
//! ### Additions
//! * `get_state_and_base()` – return `( &State, &BaseState )` by name, used by the
//!   orchestration engine to fetch common mapping definitions.
//! * derives `Eq` (helps with unit tests) and keeps existing `Clone / Debug / Serialize / Deserialize`.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::state::{base::BaseState, State};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDSL {
    #[serde(default)]
    pub comment: Option<String>,

    #[serde(default)]
    pub version: Option<String>,

    /// Name of the first state to execute
    pub start_at: String,

    /// Optional global config object
    #[serde(default)]
    pub global_config: Option<Value>,

    /// Optional error-handling definition (future work)
    #[serde(default)]
    pub error_handling: Option<Value>,

    /// All states keyed by state name
    pub states: HashMap<String, State>,
}

impl WorkflowDSL {
    /// Helper: by state name return `( &StateEnum, &BaseState )`.
    ///
    /// Engine uses this to retrieve mapping definitions (`input_mapping` / `output_mapping`)
    /// that sit inside `BaseState` without pattern-matching in multiple places.
    pub fn get_state_and_base(
        &self,
        name: &str,
    ) -> (&State, &BaseState) {
        let st = self.states.get(name).expect("state not found");
        let base = match st {
            State::Task(t)     => &t.base,
            State::Pass(p)     => &p.base,
            State::Wait(w)     => &w.base,
            State::Choice(c)   => &c.base,
            State::Succeed(s)  => &s.base,
            State::Fail(f)     => &f.base,
            State::Parallel(p) => &p.base,
            State::Map(m)      => &m.base,
        };
        (st, base)
    }
}

impl Default for BaseState {
    fn default() -> Self {
        Self {
            comment: None,
            input_mapping: None,
            output_mapping: None,
            retry: None,
            catch: None,
            next: None,
            end: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{BaseState, PassState, State};
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_workflow_dsl_serde() {
        let mut states = HashMap::new();
        states.insert(
            "step1".to_string(),
            State::Pass(PassState {
                base: BaseState {
                    comment: Some("first step".to_string()),
                    ..Default::default()
                },
                result: None,
                result_path: None,
            }),
        );
        let dsl = WorkflowDSL {
            comment: Some("test".to_string()),
            version: Some("1.0".to_string()),
            start_at: "step1".to_string(),
            global_config: Some(json!({"foo": 1})),
            error_handling: None,
            states,
        };
        let ser = serde_json::to_string(&dsl).unwrap();
        let de: WorkflowDSL = serde_json::from_str(&ser).unwrap();
        assert_eq!(de.start_at, "step1");
        assert!(de.states.contains_key("step1"));
        assert_eq!(de.comment.as_deref(), Some("test"));
        assert_eq!(de.version.as_deref(), Some("1.0"));
        assert_eq!(de.global_config.as_ref().unwrap()["foo"], 1);
    }

    #[test]
    fn test_workflow_dsl_default_roundtrip() {
        let dsl = WorkflowDSL {
            comment: None,
            version: None,
            start_at: "start".to_string(),
            global_config: None,
            error_handling: None,
            states: HashMap::new(),
        };
        let ser = serde_json::to_string(&dsl).unwrap();
        let de: WorkflowDSL = serde_json::from_str(&ser).unwrap();
        assert_eq!(de.start_at, "start");
        assert!(de.states.is_empty());
    }

    #[test]
    fn test_state_enum_serde() {
        let pass = State::Pass(PassState {
            base: BaseState::default(),
            result: None,
            result_path: None,
        });
        let ser = serde_json::to_string(&pass).unwrap();
        let de: State = serde_json::from_str(&ser).unwrap();
        match de {
            State::Pass(_) => {}
            _ => panic!("not pass state"),
        }
    }

    #[test]
    fn test_base_state_mapping_fields() {
        let base = BaseState {
            comment: Some("cmt".to_string()),
            input_mapping: None,
            output_mapping: None,
            retry: None,
            catch: None,
            next: Some("next".to_string()),
            end: Some(true),
        };
        let ser = serde_json::to_string(&base).unwrap();
        let de: BaseState = serde_json::from_str(&ser).unwrap();
        assert_eq!(de.comment.as_deref(), Some("cmt"));
        assert_eq!(de.next.as_deref(), Some("next"));
        assert_eq!(de.end, Some(true));
    }

    #[test]
    fn test_state_enum_all_variants_serde() {
        use crate::state::*;
        use crate::branch::Branch;
        use std::collections::HashMap;
        let variants: Vec<State> = vec![
            State::Task(TaskState {
                base: BaseState::default(),
                resource: "res".to_string(),
                parameters: None,
                execution_config: None,
                heartbeat_seconds: None,
                heartbeat_expr: None,
            }),
            State::Pass(PassState {
                base: BaseState::default(),
                result: None,
                result_path: None,
            }),
            State::Wait(WaitState {
                base: BaseState::default(),
                seconds: None,
                timestamp: None,
            }),
            State::Choice(ChoiceState {
                base: BaseState::default(),
                choices: vec![],
                default_next: None,
            }),
            State::Succeed(SucceedState {
                base: BaseState::default(),
            }),
            State::Fail(FailState {
                base: BaseState::default(),
                error: None,
                cause: None,
            }),
            State::Parallel(ParallelState {
                base: BaseState::default(),
                branches: vec![],
                max_concurrency: None,
            }),
            State::Map(MapState {
                base: BaseState::default(),
                items_path: "$.items".to_string(),
                iterator: Branch {
                    start_at: "step".to_string(),
                    states: HashMap::new(),
                },
                max_concurrency: None,
            }),
        ];
        for state in variants {
            let ser = serde_json::to_string(&state).unwrap();
            let de: State = serde_json::from_str(&ser).unwrap();
            // 只要能 roundtrip 就说明 serde 没问题
            match (&state, &de) {
                (State::Task(_), State::Task(_)) => {}
                (State::Pass(_), State::Pass(_)) => {}
                (State::Wait(_), State::Wait(_)) => {}
                (State::Choice(_), State::Choice(_)) => {}
                (State::Succeed(_), State::Succeed(_)) => {}
                (State::Fail(_), State::Fail(_)) => {}
                (State::Parallel(_), State::Parallel(_)) => {}
                (State::Map(_), State::Map(_)) => {}
                _ => panic!("variant mismatch: {:?} vs {:?}", state, de),
            }
        }
    }
}
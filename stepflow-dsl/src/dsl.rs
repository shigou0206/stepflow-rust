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
#[serde(rename_all = "PascalCase")]
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
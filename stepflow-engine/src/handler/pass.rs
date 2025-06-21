use async_trait::async_trait;
use serde_json::Value;
use tracing::debug;

use stepflow_dsl::state::State;
use super::{StateHandler, StateExecutionScope, StateExecutionResult};

/// ---------------------------------------------------------------------
/// PassHandler（无状态）
/// ---------------------------------------------------------------------
pub struct PassHandler;

impl PassHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl StateHandler for PassHandler {
    async fn handle(
        &self,
        scope: &StateExecutionScope<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Pass(ref s) => s,
            _ => return Err("Invalid state type for PassHandler".into()),
        };

        debug!(run_id = scope.run_id, state = scope.state_name, "PassHandler pass-through");

        Ok(StateExecutionResult {
            output: input.clone(), // 直接返回已映射后的 input 作为 output
            next_state: state.base.next.clone(),
            should_continue: true,
            metadata: None,
        })
    }

    fn state_type(&self) -> &'static str {
        "pass"
    }

    async fn on_subflow_finished(
        &self,
        _scope: &StateExecutionScope<'_>,
        _parent_context: &Value,
        _child_run_id: &str,
        _result: &Value,
    ) -> Result<StateExecutionResult, String> {
        Err("on_subflow_finished not supported by this state".into())
    }
}
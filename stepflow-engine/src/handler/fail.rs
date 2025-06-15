use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{error, debug};

use stepflow_dsl::state::{fail::FailState, State};
use super::{StateHandler, StateExecutionScope, StateExecutionResult};

/// ---------------------------------------------------------------------
/// FailHandler（无状态单例）
/// ---------------------------------------------------------------------
pub struct FailHandler;

impl FailHandler {
    pub fn new() -> Self {
        Self
    }

    fn build_error_output(&self, state: &FailState) -> Value {
        json!({
            "error": state.error,
            "cause": state.cause
        })
    }
}

#[async_trait]
impl StateHandler for FailHandler {
    async fn handle(
        &self,
        scope: &StateExecutionScope<'_>,
        _input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Fail(ref s) => s,
            _ => return Err("Invalid state type for FailHandler".into()),
        };

        error!(
            "Workflow failed with error: {:?}, cause: {:?}",
            state.error,
            state.cause
        );

        let output = self.build_error_output(state);
        debug!("Generated error output: {:?}", output);

        Ok(StateExecutionResult {
            output,
            next_state: None,            // Fail 状态是终止状态
            should_continue: false,     // 不再推进
            metadata: None,
        })
    }

    fn state_type(&self) -> &'static str {
        "fail"
    }
}
use async_trait::async_trait;
use serde_json::Value;
use tracing::debug;

use stepflow_dsl::state::State;
use crate::mapping::MappingPipeline;
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

        let pipeline = MappingPipeline {
            input_mapping: state.base.input_mapping.as_ref(),
            output_mapping: state.base.output_mapping.as_ref(),
        };

        let exec_input = pipeline.apply_input(input)?;
        let output = pipeline.apply_output(&exec_input, input)?;

        debug!("PassHandler output: {}", output);

        Ok(StateExecutionResult {
            output,
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
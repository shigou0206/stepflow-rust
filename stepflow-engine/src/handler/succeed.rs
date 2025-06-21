use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, info};

use stepflow_dsl::state::State;
use crate::mapping::MappingPipeline;
use super::{StateHandler, StateExecutionScope, StateExecutionResult};

/// ---------------------------------------------------------------------
/// SucceedHandler（无状态、终止型状态）
/// ---------------------------------------------------------------------
pub struct SucceedHandler;

impl SucceedHandler {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl StateHandler for SucceedHandler {
    async fn handle(
        &self,
        scope: &StateExecutionScope<'_>,
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Succeed(ref s) => s,
            _ => return Err("Invalid state type for SucceedHandler".into()),
        };

        let pipeline = MappingPipeline {
            input_mapping: state.base.input_mapping.as_ref(),
            output_mapping: state.base.output_mapping.as_ref(),
        };

        let exec_input = pipeline.apply_input(&scope.context)?;
        debug!(run_id = scope.run_id, state = scope.state_name, ?exec_input, "SucceedHandler input mapped");

        let final_output = pipeline.apply_output(&exec_input, &scope.context)?;
        debug!(run_id = scope.run_id, state = scope.state_name, ?final_output, "SucceedHandler output mapped");

        info!(run_id = scope.run_id, "✅ Workflow completed successfully");

        Ok(StateExecutionResult {
            output: final_output,
            next_state: None,
            should_continue: false,
            metadata: None,
        })
    }

    fn state_type(&self) -> &'static str {
        "succeed"
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
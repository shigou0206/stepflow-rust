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
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Succeed(ref s) => s,
            _ => return Err("Invalid state type for SucceedHandler".into()),
        };

        info!("✅ Workflow completed successfully");

        let pipeline = MappingPipeline {
            input_mapping: state.base.input_mapping.as_ref(),
            output_mapping: state.base.output_mapping.as_ref(),
        };

        let exec_input = pipeline.apply_input(input)?;
        let final_output = pipeline.apply_output(&exec_input, input)?;

        debug!("✅ Final context after mapping: {}", final_output);

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
}
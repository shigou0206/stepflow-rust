use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, info};
use stepflow_dsl::state::succeed::SucceedState;

use crate::mapping::MappingPipeline;
use super::{StateHandler, StateExecutionScope, StateExecutionResult};

pub struct SucceedHandler<'a> {
    state: &'a SucceedState,
}

impl<'a> SucceedHandler<'a> {
    pub fn new(state: &'a SucceedState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl<'a> StateHandler for SucceedHandler<'a> {
    async fn handle(
        &self,
        _scope: &StateExecutionScope<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        info!("✅ Workflow completed successfully");

        let pipeline = MappingPipeline {
            input_mapping: self.state.base.input_mapping.as_ref(),
            output_mapping: self.state.base.output_mapping.as_ref(),
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

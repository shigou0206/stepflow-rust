use async_trait::async_trait;
use serde_json::Value;
use tracing::debug;
use stepflow_dsl::state::pass::PassState;

use crate::mapping::MappingPipeline;
use super::{StateHandler, StateExecutionScope, StateExecutionResult};

pub struct PassHandler<'a> {
    state: &'a PassState,
}

impl<'a> PassHandler<'a> {
    pub fn new(state: &'a PassState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl<'a> StateHandler for PassHandler<'a> {
    async fn handle(
        &self,
        _scope: &StateExecutionScope<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let pipeline = MappingPipeline {
            input_mapping: self.state.base.input_mapping.as_ref(),
            output_mapping: self.state.base.output_mapping.as_ref(),
        };

        // Pass 节点只执行 input→output 映射，中间不做任何处理
        let exec_input = pipeline.apply_input(input)?;
        let output = pipeline.apply_output(&exec_input, input)?;

        debug!("PassHandler output: {}", output);

        Ok(StateExecutionResult {
            output,
            next_state: self.state.base.next.clone(),
            should_continue: true,
            metadata: None,
        })
    }

    fn state_type(&self) -> &'static str {
        "pass"
    }
}

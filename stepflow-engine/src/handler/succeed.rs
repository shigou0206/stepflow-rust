use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, info};
use std::sync::Arc;

use stepflow_dsl::state::succeed::SucceedState;
use stepflow_hook::EngineEventDispatcher;
use stepflow_storage::db::DynPM;

use crate::mapping::MappingPipeline;
use super::{StateHandler, StateExecutionContext, StateExecutionResult};

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
        _ctx: &StateExecutionContext<'_>,
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
        })
    }

    fn state_type(&self) -> &'static str {
        "succeed"
    }
}

/// 兼容调用形式：传入已有 SucceedState
pub async fn handle_succeed(
    state_name: &str,
    state: &SucceedState,
    input: &Value,
    run_id: &str,
    event_dispatcher: &Arc<EngineEventDispatcher>,
    persistence: &DynPM,
) -> Result<Value, String> {
    let ctx = StateExecutionContext::new(
        run_id,
        state_name,
        "succeed",
        crate::engine::WorkflowMode::Inline, // 或者从外部传入
        event_dispatcher,
        persistence,
    );

    let handler = SucceedHandler::new(state);
    let result = handler.execute(&ctx, input).await?;
    Ok(result.output)
}
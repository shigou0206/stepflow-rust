use async_trait::async_trait;
use serde_json::Value;
use tracing::debug;
use stepflow_dsl::state::pass::PassState;
use stepflow_hook::EngineEventDispatcher;
use std::sync::Arc;
use stepflow_storage::persistence_manager::PersistenceManager;

use crate::mapping::MappingPipeline;
use super::{StateHandler, StateExecutionContext, StateExecutionResult};

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
        _ctx: &StateExecutionContext<'_>,
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
        })
    }

    fn state_type(&self) -> &'static str {
        "pass"
    }
}

/// 向后兼容的函数形式
pub async fn handle_pass(
    state_name: &str,
    state: &PassState,
    input: &Value,
    run_id: &str,
    event_dispatcher: &Arc<EngineEventDispatcher>,
    persistence: &Arc<dyn PersistenceManager>,
) -> Result<Value, String> {
    let ctx = StateExecutionContext::new(
        run_id,
        state_name,
        "pass",
        crate::engine::WorkflowMode::Inline,
        event_dispatcher,
        persistence,
    );

    println!("handle_pass input: {}", input);

    let handler = PassHandler::new(state);
    let result = handler.execute(&ctx, input).await?;
    Ok(result.output)
}
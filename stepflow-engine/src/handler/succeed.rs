use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, info};
use std::sync::Arc;

use stepflow_dsl::state::{succeed::SucceedState, BaseState};
use stepflow_storage::persistence_manager::PersistenceManager;
use stepflow_hook::EngineEventDispatcher;

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
            parameter_mapping: self.state.base.parameter_mapping.as_ref(),
            output_mapping: self.state.base.output_mapping.as_ref(),
        };

        let exec_input = pipeline.apply_input(input)?;
        let _param = pipeline.apply_parameter(&exec_input)?;
        let final_output = pipeline.apply_output(&exec_input, &exec_input)?;

        debug!("✅ Final context after mapping: {:?}", final_output);

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

pub async fn handle_succeed(
    state_name: &str,
    input: &Value,
    run_id: &str,
    event_dispatcher: &Arc<EngineEventDispatcher>,
    persistence: &Arc<dyn PersistenceManager>,
) -> Result<Value, String> {
    let ctx = StateExecutionContext::new(
        run_id,
        state_name,
        "succeed", // ✅ 添加 state_type
        crate::engine::WorkflowMode::Inline,
        event_dispatcher,
        persistence,
    );

    let state = SucceedState {
        base: BaseState {
            comment: None,
            input_mapping: None,
            parameter_mapping: None,
            output_mapping: None,
            retry: None,
            catch: None,
            next: None,
            end: Some(true),
        },
    };

    let handler = SucceedHandler::new(&state);
    let result = handler.execute(&ctx, input).await?;
    Ok(result.output)
}
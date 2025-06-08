use async_trait::async_trait;
use serde_json::{Value, Map};
use tracing::{debug, error};
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

    fn merge(&self, input: &Value, param: &Value) -> Result<Value, String> {
        match (input, param) {
            (Value::Object(base), Value::Object(extra)) => {
                let mut merged = Map::new();
                for (k, v) in base.iter() {
                    merged.insert(k.clone(), v.clone());
                }
                for (k, v) in extra.iter() {
                    merged.insert(k.clone(), v.clone());
                }
                Ok(Value::Object(merged))
            }
            _ => {
                error!("PassState merge error: input or parameters not objects");
                Err("PassState requires both input and parameters to be objects.".into())
            }
        }
    }
}

#[async_trait]
impl<'a> StateHandler for PassHandler<'a> {
    async fn handle(
        &self,
        ctx: &StateExecutionContext<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let pipeline = MappingPipeline {
            input_mapping: self.state.base.input_mapping.as_ref(),
            parameter_mapping: self.state.base.parameter_mapping.as_ref(),
            output_mapping: self.state.base.output_mapping.as_ref(),
        };

        let exec_input = pipeline.apply_input(input)?;
        let param = pipeline.apply_parameter(&exec_input)?;
        let merged = self.merge(&exec_input, &param)?;
        let final_output = pipeline.apply_output(&merged, &exec_input)?;

        debug!("PassHandler output: {}", final_output);

        Ok(StateExecutionResult {
            output: final_output,
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
        "pass", // ✅ 添加 state_type
        crate::engine::WorkflowMode::Inline,
        event_dispatcher,
        persistence,
    );

    let handler = PassHandler::new(state);
    let result = handler.execute(&ctx, input).await?;
    Ok(result.output)
}
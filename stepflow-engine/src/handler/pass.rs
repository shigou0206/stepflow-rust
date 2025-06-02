use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;
use tracing::debug;
use stepflow_dsl::state::pass::PassState;
use stepflow_hook::EngineEventDispatcher;
use std::sync::Arc;
use super::{StateHandler, StateExecutionContext, StateExecutionResult};
use stepflow_storage::PersistenceManager;

#[derive(Error, Debug)]
pub enum PassError {
    #[error("Invalid result format: expected object, got {0}")]
    InvalidResultFormat(String),
    #[error("Invalid input format: expected object, got {0}")]
    InvalidInputFormat(String),
}

pub struct PassHandler<'a> {
    state: &'a PassState,
}

impl<'a> PassHandler<'a> {
    pub fn new(state: &'a PassState) -> Self {
        Self { state }
    }

    fn merge_result(&self, input: &Value) -> Result<Value, String> {
        // 如果没有 result，直接返回输入
        if self.state.result.is_none() {
            debug!("No result specified, passing through input");
            return Ok(input.clone());
        }

        let result = self.state.result.as_ref().unwrap();
        
        // 验证 result 格式
        let result_obj = match result {
            Value::Object(map) => map,
            _ => {
                let err = PassError::InvalidResultFormat(result.to_string());
                debug!("Invalid result format: {}", err);
                return Err(err.to_string());
            }
        };

        // 验证输入格式
        let mut output = match input {
            Value::Object(map) => Value::Object(map.clone()),
            _ => {
                let err = PassError::InvalidInputFormat(input.to_string());
                debug!("Invalid input format: {}", err);
                return Err(err.to_string());
            }
        };

        // 合并结果
        if let Value::Object(ref mut out_map) = output {
            for (k, v) in result_obj {
                debug!("Merging key '{}' from result", k);
                out_map.insert(k.clone(), v.clone());
            }
        }

        debug!("Successfully merged result into input");
        Ok(output)
    }
}

#[async_trait(?Send)]
impl<'a> StateHandler for PassHandler<'a> {
    async fn handle(
        &self,
        _ctx: &StateExecutionContext<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let output = self.merge_result(input)?;

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

/// Pass handler – 合并 `result` 字段到输入（若缺省则返回输入）。
///
/// # 处理规则
/// 1. 如果 `result` 不存在，直接返回输入
/// 2. 如果 `result` 存在且为对象，将其合并到输入中
/// 3. 如果输入不是对象，返回错误
/// 4. 如果 `result` 不是对象，返回错误
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
        crate::engine::WorkflowMode::Inline, // Pass 不区分模式
        event_dispatcher,
        persistence,
    );

    let handler = PassHandler::new(state);
    let result = handler.execute(&ctx, input).await?;
    
    Ok(result.output)
}
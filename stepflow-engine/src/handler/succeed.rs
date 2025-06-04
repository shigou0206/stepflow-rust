use async_trait::async_trait;
use serde_json::Value;
use tracing::{debug, info};
use stepflow_dsl::state::succeed::SucceedState;
use super::{StateHandler, StateExecutionContext, StateExecutionResult};
use std::sync::Arc;
use stepflow_storage::persistence_manager::PersistenceManager;
use stepflow_hook::EngineEventDispatcher;

/// Succeed handler - 工作流成功结束处理器
///
/// # 功能说明
/// - 将工作流的最终上下文原样返回
/// - 不对上下文做任何修改
/// - Engine 收到后会将工作流置为 Completed 状态
///
/// # 返回值
/// - `Ok(Value)`: 原始上下文的克隆
/// - `Err`: 永不返回错误
///
/// # 注意事项
/// - 这是一个终止状态，之后的状态不会被执行
/// - 即使上下文为空也会正常返回
pub struct SucceedHandler;

impl SucceedHandler {
    pub fn new(_state: &SucceedState) -> Self {
        Self
    }
}

#[async_trait]
impl<'a> StateHandler for SucceedHandler {
    async fn handle(
        &self,
        _ctx: &StateExecutionContext<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        info!("Workflow completed successfully");
        debug!("Final context: {:?}", input);

        Ok(StateExecutionResult {
            output: input.clone(),
            next_state: None,  // Succeed 状态是终止状态
            should_continue: false,  // 工作流将终止
        })
    }

    fn state_type(&self) -> &'static str {
        "succeed"
    }
}

// 为了保持向后兼容，保留原有的函数签名
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
        crate::engine::WorkflowMode::Inline,
        event_dispatcher,
        persistence,
    );

    let handler = SucceedHandler::new(&SucceedState {
        base: stepflow_dsl::state::BaseState {
            next: None,
            comment: None,
            input_mapping: None,
            output_mapping: None,
            retry: None,
            catch: None,
            end: Some(true),  // Succeed 状态总是终止状态
        },
    });
    let result = handler.execute(&ctx, input).await?;
    
    Ok(result.output)
}
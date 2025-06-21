use async_trait::async_trait;
use super::execution_scope::{StateExecutionScope, StateExecutionResult};

/// 统一的状态处理器接口
#[async_trait]
pub trait StateHandler: Send + Sync {
    /// 处理状态逻辑
    async fn handle(
        &self,
        scope: &StateExecutionScope<'_>,
    ) -> Result<StateExecutionResult, String>;

    /// 获取状态类型
    fn state_type(&self) -> &'static str;

    /// 子流程完成后的事件处理（Map/Parallel 专用）
    async fn on_subflow_finished(
        &self,
        scope: &StateExecutionScope<'_>,
        parent_context: &serde_json::Value,
        child_run_id: &str,
        result: &serde_json::Value,
    ) -> Result<StateExecutionResult, String>;
}
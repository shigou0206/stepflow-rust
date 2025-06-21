use async_trait::async_trait;
use serde_json::Value;
use tracing::info;

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
        _scope: &StateExecutionScope<'_>,
        input: &Value, // ✅ 已是 input_mapping 处理后的输入
    ) -> Result<StateExecutionResult, String> {
        let output = input.clone(); // ✅ handler 只需原样返回

        info!("✅ Workflow completed successfully");

        Ok(StateExecutionResult {
            output,
            next_state: None,          // ✅ 终止型节点
            should_continue: false,    // ✅ 不再推进
            metadata: None,
        })
    }

    fn state_type(&self) -> &'static str {
        "succeed"
    }

    async fn on_subflow_finished(
        &self,
        _scope: &StateExecutionScope<'_>,
        _parent_context: &Value,
        _child_run_id: &str,
        _result: &Value,
    ) -> Result<StateExecutionResult, String> {
        Err("on_subflow_finished not supported by this state".into())
    }
}
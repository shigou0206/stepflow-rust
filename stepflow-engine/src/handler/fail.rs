use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{debug, error};

use stepflow_dsl::state::{fail::FailState, State};
use crate::mapping::MappingPipeline;
use super::{StateHandler, StateExecutionScope, StateExecutionResult};

/// ---------------------------------------------------------------------
/// FailHandler（终止型节点，处理失败信息）
/// ---------------------------------------------------------------------
pub struct FailHandler;

impl FailHandler {
    pub fn new() -> Self {
        Self
    }

    fn build_error_output(&self, state: &FailState) -> Value {
        json!({
            "error": state.error,
            "cause": state.cause
        })
    }
}

#[async_trait]
impl StateHandler for FailHandler {
    async fn handle(
        &self,
        scope: &StateExecutionScope<'_>,
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Fail(ref s) => s,
            _ => return Err("Invalid state type for FailHandler".into()),
        };

        let pipeline = MappingPipeline {
            input_mapping: state.base.input_mapping.as_ref(),
            output_mapping: state.base.output_mapping.as_ref(),
        };

        // ✅ 映射输入（虽然通常无用，但为了统一）
        let exec_input = pipeline.apply_input(&scope.context)?;
        debug!(run_id = scope.run_id, state = scope.state_name, ?exec_input, "FailHandler input mapped");

        // ⚠️ 失败节点强制生成 error 输出（覆盖一切）
        let raw_output = self.build_error_output(state);

        // ✅ 输出映射（如需包装失败信息到 context 中）
        let final_output = pipeline.apply_output(&raw_output, &scope.context)?;
        debug!(run_id = scope.run_id, state = scope.state_name, ?final_output, "FailHandler output mapped");

        error!(
            run_id = scope.run_id,
            state = scope.state_name,
            "❌ Workflow failed: {:?} | cause: {:?}",
            state.error, state.cause
        );

        Ok(StateExecutionResult {
            output: final_output,
            next_state: None,           // 终止节点
            should_continue: false,     // 停止推进
            metadata: None,
        })
    }

    fn state_type(&self) -> &'static str {
        "fail"
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
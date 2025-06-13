//! Fail handler – 构造 `{ error, cause }` JSON 并原样返回。
//!
//! * 工作流在 Engine 层收到后会进入终止状态，
//!   因此这里永远 `should_continue = false`，
//!   但新接口只需返回业务 `Value`。

use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::{error, debug};
use stepflow_dsl::state::fail::FailState;
use super::{StateHandler, StateExecutionScope, StateExecutionResult};

pub struct FailHandler<'a> {
    state: &'a FailState,
}

impl<'a> FailHandler<'a> {
    pub fn new(state: &'a FailState) -> Self {
        Self { state }
    }

    fn build_error_output(&self) -> Value {
        json!({
            "error": self.state.error,
            "cause": self.state.cause
        })
    }
}

#[async_trait]
impl<'a> StateHandler for FailHandler<'a> {
    async fn handle(
        &self,
        _scope: &StateExecutionScope<'_>,
        _input: &Value,
    ) -> Result<StateExecutionResult, String> {
        error!(
            "Workflow failed with error: {:?}, cause: {:?}",
            self.state.error,
            self.state.cause
        );

        let output = self.build_error_output();
        debug!("Generated error output: {:?}", output);

        Ok(StateExecutionResult {
            output,
            next_state: None,              // Fail 状态是终止状态
            should_continue: false,       // 工作流将终止
            metadata: None,
        })
    }

    fn state_type(&self) -> &'static str {
        "fail"
    }
}

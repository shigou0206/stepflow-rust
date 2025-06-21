use crate::{
    command::Command,
    handler::registry::StateHandlerRegistry,
    handler::execution_scope::{StateExecutionScope, StateExecutionResult},
};
use serde_json::Value;
use stepflow_dsl::State;
use stepflow_storage::db::DynPM;
use super::types::StepOutcome;

/// 调度失败类型
#[derive(thiserror::Error, Debug)]
pub enum DispatchError {
    #[error("Task execution failed: {0}")]
    TaskError(String),
    #[error("State transition failed: {0}")]
    StateError(String),
}

impl From<String> for DispatchError {
    fn from(s: String) -> Self {
        DispatchError::StateError(s)
    }
}

impl From<DispatchError> for String {
    fn from(err: DispatchError) -> Self {
        err.to_string()
    }
}

/// WorkflowEngine 调用的统一状态执行入口（无事件）
/// 返回：(StepOutcome, Option<next_state>, raw_output, metadata)
pub(crate) async fn dispatch_command(
    cmd: &Command,
    state_enum: &State,
    context: &Value,
    run_id: &str,
    persistence: &DynPM,
    registry: &StateHandlerRegistry,
) -> Result<(StepOutcome, Option<String>, Value, Option<Value>), String> {
    let state_name = cmd.state_name().to_string();
    let state_type = state_enum.variant_name();

    let handler = registry
        .get(state_type)
        .ok_or_else(|| format!("No handler registered for state type: {state_type}"))?;

    // 构造 StateExecutionScope（传递 context）
    let scope = StateExecutionScope::new(
        run_id,
        &state_name,
        state_type,
        None,
        persistence,
        state_enum,
        context,
    );

    // 调用对应的 handler
    let result: StateExecutionResult = handler
        .handle(&scope)
        .await
        .map_err(|e| DispatchError::StateError(e.to_string()))?;

    // 特殊处理 Choice 的 next_state
    let logical_next = if let (Command::Choice { next_state, .. }, State::Choice(_)) = (cmd, state_enum) {
        Some(next_state.clone())
    } else {
        result.next_state.clone()
    };

    // 返回 StepOutcome
    Ok((
        StepOutcome {
            should_continue: logical_next.is_some(),
            updated_context: result.output.clone(),
            is_blocking: result.is_blocking,
        },
        logical_next,
        result.output,
        result.metadata,
    ))
}
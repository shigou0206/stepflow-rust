// ✅ dispatch_command：支持 Map / Parallel 状态
use crate::{
    command::Command,
    handler::registry::StateHandlerRegistry,
    mapping::MappingPipeline,
};
use serde_json::Value;
use stepflow_dsl::{state::base::BaseState, State};
use stepflow_storage::db::DynPM;
use super::types::{StepOutcome, WorkflowMode};
use crate::handler::execution_scope::StateExecutionScope;

/// 调度失败类型
#[derive(thiserror::Error, Debug)]
pub enum DispatchError {
    #[error("Task execution failed: {0}")]
    TaskError(String),
    #[error("State transition failed: {0}")]
    StateError(String),
    #[error("Input/Output mapping failed: {0}")]
    MappingError(String),
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
    mode: WorkflowMode,
    persistence: &DynPM,
    registry: &StateHandlerRegistry,
) -> Result<(StepOutcome, Option<String>, Value, Option<Value>), String> {
    let state_name = cmd.state_name().to_string();
    let state_type = state_enum.variant_name();

    // ---------- 1. 提取基础状态定义 ----------
    let base: &BaseState = match state_enum {
        State::Task(s) => &s.base,
        State::Wait(s) => &s.base,
        State::Pass(s) => &s.base,
        State::Choice(s) => &s.base,
        State::Fail(s) => &s.base,
        State::Succeed(s) => &s.base,
        State::Parallel(s) => &s.base,
        State::Map(s) => &s.base,
    };

    // ---------- 2. 执行输入映射 ----------
    let pipeline = MappingPipeline {
        input_mapping: base.input_mapping.as_ref(),
        output_mapping: base.output_mapping.as_ref(),
    };

    let exec_in = pipeline
        .apply_input(context)
        .map_err(|e| format!("apply_input failed: {e:?}"))?;

    // ---------- 3. 查找 handler 并执行 ----------
    let handler = registry
        .get(state_type)
        .ok_or_else(|| format!("No handler registered for state type: {state_type}"))?;

    let scope = StateExecutionScope::new(
        run_id,
        &state_name,
        state_type,
        mode,
        None,
        persistence,
        state_enum,
        context,
    );

    let result = handler
        .handle(&scope, &exec_in)
        .await
        .map_err(|e| DispatchError::StateError(e.to_string()))?;

    // ---------- 4. Choice 特殊处理 ----------
    let logical_next = if let (Command::Choice { next_state, .. }, State::Choice(_)) = (cmd, state_enum) {
        Some(next_state.clone())
    } else {
        result.next_state.clone()
    };

    // ---------- 5. 执行输出映射 ----------
    let new_ctx = pipeline
        .apply_output(&result.output, context)
        .map_err(|e| DispatchError::MappingError(e.to_string()))?;

    Ok((
        StepOutcome {
            should_continue: logical_next.is_some(),
            updated_context: new_ctx,
        },
        logical_next,
        result.output,
        result.metadata,
    ))
}

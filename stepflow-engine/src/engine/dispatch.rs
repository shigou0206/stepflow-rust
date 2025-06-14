//! dispatch.rs —— 纯粹状态执行器，不负责事件、不做副作用。
use crate::{command::Command, handler, mapping::MappingPipeline};
use serde_json::Value;
use std::sync::Arc;
use stepflow_dsl::{state::base::BaseState, State};
use stepflow_match::service::MatchService;
use stepflow_storage::db::DynPM;
use super::types::{StepOutcome, WorkflowMode};
use crate::handler::traits::StateHandler;

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
    match_service: Arc<dyn MatchService>,
    persistence: &DynPM,
) -> Result<(StepOutcome, Option<String>, Value, Option<Value>), String> {
    let state_name = cmd.state_name().to_string();

    // ---------- 1. 提取基础状态定义 ----------
    let base: &BaseState = match state_enum {
        State::Task(s) => &s.base,
        State::Wait(s) => &s.base,
        State::Pass(s) => &s.base,
        State::Choice(s) => &s.base,
        State::Fail(s) => &s.base,
        State::Succeed(s) => &s.base,
        State::Parallel(_) | State::Map(_) => {
            return Err("Parallel / Map not yet supported".to_string());
        }
    };

    // ---------- 2. 执行输入映射 ----------
    let pipeline = MappingPipeline {
        input_mapping: base.input_mapping.as_ref(),
        output_mapping: base.output_mapping.as_ref(),
    };

    let exec_in = pipeline
        .apply_input(context)
        .map_err(|e| format!("apply_input failed: {e:?}"))?;

    // ---------- 3. 执行状态本体 ----------
    let (raw_out, logical_next, metadata): (Value, Option<String>, Option<Value>) = match state_enum {
        State::Task(t) => {
            let handler = handler::TaskHandler::new(match_service.clone(), t);
            let scope = handler::StateExecutionScope::new(
                run_id, &state_name, "task", mode, None, persistence,
            );
            let result = handler.handle(&scope, &exec_in).await.map_err(|e| DispatchError::TaskError(e.to_string()))?;
            (result.output, result.next_state, result.metadata)
        }

        State::Wait(w) => {
            let handler = handler::WaitHandler::new(w);
            let scope = handler::StateExecutionScope::new(
                run_id, &state_name, "wait", mode, None, persistence,
            );
            let result = handler.handle(&scope, &exec_in).await.map_err(|e| DispatchError::StateError(e.to_string()))?;
            (result.output, result.next_state, result.metadata)
        }

        State::Pass(p) => {
            let handler = handler::PassHandler::new(p);
            let scope = handler::StateExecutionScope::new(
                run_id, &state_name, "pass", mode, None, persistence,
            );
            let result = handler.handle(&scope, &exec_in).await.map_err(|e| DispatchError::StateError(e.to_string()))?;
            (result.output, result.next_state, result.metadata)
        }

        State::Choice(c) => {
            let handler = handler::ChoiceHandler::new(c);
            let scope = handler::StateExecutionScope::new(
                run_id, &state_name, "choice", mode, None, persistence,
            );
            let result = handler.handle(&scope, &exec_in).await.map_err(|e| DispatchError::StateError(e.to_string()))?;
            (result.output, result.next_state, result.metadata)
        }

        State::Succeed(s) => {
            let handler = handler::SucceedHandler::new(s);
            let scope = handler::StateExecutionScope::new(
                run_id, &state_name, "succeed", mode, None, persistence,
            );
            let result = handler.handle(&scope, &exec_in).await.map_err(|e| DispatchError::StateError(e.to_string()))?;
            (result.output, result.next_state, result.metadata)
        }

        State::Fail(f) => {
            let handler = handler::FailHandler::new(f);
            let scope = handler::StateExecutionScope::new(
                run_id, &state_name, "fail", mode, None, persistence,
            );
            let result = handler.handle(&scope, &exec_in).await.map_err(|e| DispatchError::StateError(e.to_string()))?;
            (result.output, result.next_state, result.metadata)
        }

        _ => {
            let err = "Parallel / Map not supported".to_string();
            return Err(DispatchError::StateError(err).to_string());
        }
    };

    // ---------- 4. Choice 特殊处理 ----------
    let logical_next = if let (Command::Choice { next_state, .. }, State::Choice(_)) = (cmd, state_enum) {
        Some(next_state.clone())
    } else {
        logical_next
    };

    // ---------- 5. 执行输出映射 ----------
    let new_ctx = pipeline
        .apply_output(&raw_out, context)
        .map_err(|e| DispatchError::MappingError(e.to_string()))?;

    Ok( (
        StepOutcome {
            should_continue: logical_next.is_some(),
            updated_context: new_ctx,
        },
        logical_next,
        raw_out,
        metadata,
    ))
}
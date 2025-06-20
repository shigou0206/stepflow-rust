use serde_json::Value;
use std::sync::Arc;

use stepflow_hook::EngineEventDispatcher;
use stepflow_storage::db::DynPM;
use stepflow_dsl::State;

/// ------------------------------------------------------------
/// StateExecutionResult —— handler 的统一输出
/// ------------------------------------------------------------
#[derive(Debug)]
pub struct StateExecutionResult {
    pub output: Value,
    pub next_state: Option<String>,
    pub should_continue: bool,
    pub metadata: Option<Value>, 
}

/// ------------------------------------------------------------
/// StateExecutionContext —— handler 运行时上下文
/// ------------------------------------------------------------
pub struct StateExecutionScope<'a> {
    pub run_id: &'a str,
    pub state_name: &'a str,
    pub state_type: &'a str,
    pub dispatcher: Option<&'a Arc<EngineEventDispatcher>>,
    pub persistence: &'a DynPM,
    pub state_def: &'a State,
    pub context: &'a Value, // ✅ 添加当前上下文，便于子流程聚合等
}

impl<'a> StateExecutionScope<'a> {
    pub fn new(
        run_id: &'a str,
        state_name: &'a str,
        state_type: &'a str,
        dispatcher: Option<&'a Arc<EngineEventDispatcher>>,
        persistence: &'a DynPM,
        state_def: &'a State,
        context: &'a Value, // ✅ 新参数
    ) -> Self {
        Self {
            run_id,
            state_name,
            state_type,
            dispatcher,
            persistence,
            state_def,
            context, // ✅ 新字段赋值
        }
    }

    /// 便捷方法：提取 next_state（用于推进）
    pub fn next(&self) -> Option<&String> {
        match self.state_def {
            State::Task(s) => s.base.next.as_ref(),
            State::Wait(s) => s.base.next.as_ref(),
            State::Pass(s) => s.base.next.as_ref(),
            State::Choice(_) => None,
            State::Succeed(_) => None,
            State::Fail(_) => None,
            State::Map(s) => s.base.next.as_ref(),
            State::Parallel(s) => s.base.next.as_ref(),
        }
    }
}
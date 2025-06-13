use serde_json::Value;
use std::sync::Arc;

use stepflow_hook::EngineEventDispatcher;
use stepflow_storage::db::DynPM;

use crate::engine::WorkflowMode;

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
    pub mode: WorkflowMode,
    pub dispatcher: Option<&'a Arc<EngineEventDispatcher>>,
    pub persistence: &'a DynPM,
}

impl<'a> StateExecutionScope<'a> {
    pub fn new(
        run_id: &'a str,
        state_name: &'a str,
        state_type: &'a str,
        mode: WorkflowMode,
        dispatcher: Option<&'a Arc<EngineEventDispatcher>>,
        persistence: &'a DynPM,
    ) -> Self {
        Self {
            run_id,
            state_name,
            state_type,
            mode,
            dispatcher,
            persistence,
        }
    }
}
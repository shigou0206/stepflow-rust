// ✅ ExecutionSignal：添加 SubflowFinished 支持
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ExecutionSignal {
    TaskCompleted {
        run_id: String,
        state_name: String,
        output: Value,
    },
    TaskFailed {
        run_id: String,
        state_name: String,
        error: String,
    },
    TaskCancelled {
        run_id: String,
        state_name: String,
        reason: Option<String>,
    },
    TimerFired {
        run_id: String,
        state_name: String,
    },
    Heartbeat {
        run_id: String,
        state_name: String,
        details: Option<Value>,
    },
    SubflowFinished {
        parent_run_id: String,
        child_run_id: String,
        state_name: String,
        result: Value,
    },
}
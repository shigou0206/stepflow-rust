use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EngineEvent {
    WorkflowStarted {
        run_id: String,
    },
    NodeEnter {
        run_id: String,
        state_name: String,
        input: Value,
    },
    NodeSuccess {
        run_id: String,
        state_name: String,
        output: Value,
    },
    NodeFailed {
        run_id: String,
        state_name: String,
        error: String,
    },
    WorkflowFinished {
        run_id: String,
        result: Value,
    },
    NodeDispatched {
        run_id: String,
        state_name: String,
        context: Value,
    },
}
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum EngineEvent {
    // === 核心流程 ===
    WorkflowStarted {
        run_id: String,
    },
    WorkflowFinished {
        run_id: String,
        result: Value,
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
    NodeCancelled {
        run_id: String,
        state_name: String,
        reason: String,
    },
    NodeExit {
        run_id: String,
        state_name: String,
        status: String, // "success", "failed", "cancelled"
        duration_ms: Option<u64>,
    },

    // === 调度相关 ===
    NodeDispatched {
        run_id: String,
        state_name: String,
        context: Value,
    },
    TimerScheduled {
        run_id: String,
        state_name: String,
        timestamp: String,
    },
    TimerFired {
        run_id: String,
        state_name: String,
    },
    TaskReady {
        run_id: String,
        state_name: String,
        resource: String,
        input: Option<Value>,
    },
    TaskFinished {
        run_id: String,
        state_name: String,
        output: Value,
    },

    SubflowReady {
        run_id: String,           // 子流程 run_id
        parent_run_id: String,    // 父流程 run_id
        state_name: String,       // 所属 Map/Parallel 状态
    },

    SubflowFinished {
        parent_run_id: String,
        child_run_id: String,
        state_name: String,
        result: Value,
    },

    SubflowFailed {
        parent_run_id: String,
        child_run_id: String,
        state_name: String,
        error: String,
    },

    // === 扩展 & UI ===
    UiEventPushed {
        run_id: String,
        ui_event: Value,
    },
}

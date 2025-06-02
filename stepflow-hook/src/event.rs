use serde_json::Value;
use chrono::NaiveDateTime;
use stepflow_engine::engine::WorkflowMode;

#[derive(Debug, Clone)]
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
    WaitStart {
        run_id: String,
        state_name: String,
        mode: WorkflowMode,
    },
    WaitComplete {
        run_id: String,
        state_name: String,
        mode: WorkflowMode,
    },
    WaitError {
        run_id: String,
        state_name: String,
        error: String,
    },
    TimerCreated {
        run_id: String,
        state_name: String,
        timer_id: String,
        fire_at: NaiveDateTime,
    },
    TimerError {
        run_id: String,
        state_name: String,
        timer_id: String,
        error: String,
    },
}
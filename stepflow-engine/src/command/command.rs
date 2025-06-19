// ✅ Command：增加支持 Map / Parallel 状态指令
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    ExecuteTask {
        state_name: String,
        resource: String,
        next_state: Option<String>,
    },
    Wait {
        state_name: String,
        seconds: u64,
        wait_until: DateTime<Utc>,
        next_state: Option<String>,
    },
    Pass {
        state_name: String,
        output: Value,
        next_state: Option<String>,
    },
    Choice {
        state_name: String,
        next_state: String,
    },
    Succeed {
        state_name: String,
        output: Value,
    },
    Fail {
        state_name: String,
        error: Option<String>,
        cause: Option<String>,
    },
    Map {
        state_name: String,
        next_state: Option<String>,
    },
    Parallel {
        state_name: String,
        next_state: Option<String>,
    },
}

impl Command {
    pub fn state_name(&self) -> &str {
        match self {
            Command::ExecuteTask { state_name, .. } |
            Command::Wait { state_name, .. } |
            Command::Pass { state_name, .. } |
            Command::Choice { state_name, .. } |
            Command::Succeed { state_name, .. } |
            Command::Fail { state_name, .. } |
            Command::Map { state_name, .. } |
            Command::Parallel { state_name, .. } => state_name
        }
    }
}
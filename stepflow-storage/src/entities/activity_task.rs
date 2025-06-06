use chrono::NaiveDateTime;
use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredActivityTask {
    pub task_token: String,
    pub run_id: String,
    pub shard_id: i64,
    pub seq: i64,
    pub activity_type: String,
    pub state_name: Option<String>,
    pub input: Option<Value>,
    pub result: Option<Value>,
    pub status: String,
    pub error: Option<String>,
    pub error_details: Option<String>,
    pub attempt: i64,
    pub max_attempts: i64,
    pub heartbeat_at: Option<NaiveDateTime>,
    pub scheduled_at: NaiveDateTime,
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub timeout_seconds: Option<i64>,
    pub retry_policy: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateStoredActivityTask {
    pub state_name: Option<String>,
    pub input: Option<Option<Value>>,
    pub result: Option<Option<Value>>,
    pub status: Option<String>,
    pub error: Option<Option<String>>,
    pub error_details: Option<Option<String>>,
    pub attempt: Option<i64>,
    pub heartbeat_at: Option<Option<NaiveDateTime>>,
    pub started_at: Option<Option<NaiveDateTime>>,
    pub completed_at: Option<Option<NaiveDateTime>>,
    pub version: Option<i64>,
}


use chrono::NaiveDateTime;
use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredWorkflowState {
    pub state_id: String,          // 改名为 state_id
    pub run_id: String,
    pub shard_id: i64,             // 添加
    pub state_name: String,
    pub state_type: String,
    pub status: String,
    pub input: Option<Value>,
    pub output: Option<Value>,
    pub error: Option<String>,     // 改名为 error
    pub error_details: Option<String>, // 添加
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>, // 改名为 completed_at
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub version: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateStoredWorkflowState {
    pub state_name: Option<String>,
    pub state_type: Option<String>,
    pub status: Option<String>,
    pub input: Option<Option<Value>>,
    pub output: Option<Option<Value>>,
    pub error: Option<Option<String>>,
    pub error_details: Option<Option<String>>,
    pub started_at: Option<Option<NaiveDateTime>>,
    pub completed_at: Option<Option<NaiveDateTime>>,
    pub version: Option<i64>,
} 
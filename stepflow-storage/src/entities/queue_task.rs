use chrono::NaiveDateTime;
use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredQueueTask {
    pub task_id: String,
    pub run_id: String,
    pub state_name: String,
    pub task_payload: Option<Value>,    // 改名为 task_payload 以匹配 SQLite 模型
    pub status: String,
    pub attempts: i64,                  // 改为 i64
    pub max_attempts: i64,              // 添加
    pub error_message: Option<String>,  // 改名为 error_message
    pub last_error_at: Option<NaiveDateTime>,
    pub next_retry_at: Option<NaiveDateTime>,
    pub queued_at: NaiveDateTime,       // 添加
    pub processing_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub failed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateStoredQueueTask {
    pub status: Option<String>,
    pub task_payload: Option<Option<Value>>,  // 改名为 task_payload
    pub attempts: Option<i64>,                // 改为 i64
    pub error_message: Option<Option<String>>, // 改名为 error_message
    pub last_error_at: Option<Option<NaiveDateTime>>,
    pub next_retry_at: Option<Option<NaiveDateTime>>,
    pub processing_at: Option<Option<NaiveDateTime>>,
    pub completed_at: Option<Option<NaiveDateTime>>,
    pub failed_at: Option<Option<NaiveDateTime>>,
} 
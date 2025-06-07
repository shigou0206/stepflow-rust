use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 数据库存储结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredQueueTask {
    pub task_id: String,
    pub run_id: String,
    pub state_name: String,
    pub resource: String,
    pub task_payload: Option<Value>,
    pub status: String,
    pub attempts: i64,
    pub max_attempts: i64,
    pub priority: Option<i64>,           
    pub timeout_seconds: Option<i64>,    
    pub error_message: Option<String>,
    pub last_error_at: Option<NaiveDateTime>,
    pub next_retry_at: Option<NaiveDateTime>,
    pub queued_at: NaiveDateTime,
    pub processing_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub failed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 可选字段更新结构
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateStoredQueueTask {
    pub status: Option<String>,
    pub task_payload: Option<Option<Value>>,
    pub attempts: Option<i64>,
    pub priority: Option<i64>,           
    pub timeout_seconds: Option<i64>,    
    pub error_message: Option<Option<String>>,
    pub last_error_at: Option<Option<NaiveDateTime>>,
    pub next_retry_at: Option<Option<NaiveDateTime>>,
    pub processing_at: Option<Option<NaiveDateTime>>,
    pub completed_at: Option<Option<NaiveDateTime>>,
    pub failed_at: Option<Option<NaiveDateTime>>,
}
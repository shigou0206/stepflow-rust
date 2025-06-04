use chrono::NaiveDateTime;
use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredTimer {
    pub timer_id: String, // Unique identifier for the timer
    pub run_id: String,   // Associated workflow execution run ID
    pub shard_id: i64,     // Added shard_id
    pub fire_at: NaiveDateTime, // Timestamp when the timer should fire
    pub status: String, // e.g., "pending", "fired", "cancelled"
    pub version: i64,
    pub state_name: Option<String>, // Optional: state that set this timer
    pub payload: Option<Value>,    // 保持 Value 类型，SQLite 层会转换为 String
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateStoredTimer {
    pub fire_at: Option<NaiveDateTime>,
    pub status: Option<String>,
    pub version: Option<i64>,
    pub payload: Option<Option<Value>>,  // 保持 Option<Option<Value>> 以支持设置为 None
} 
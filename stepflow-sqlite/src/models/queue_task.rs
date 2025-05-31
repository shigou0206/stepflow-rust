use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QueueTask {
    pub task_id: String,
    pub run_id: String,
    pub state_name: String,
    pub task_payload: Option<String>,
    pub status: String,
    pub attempts: i64,
    pub max_attempts: i64,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Default)]
pub struct UpdateQueueTask {
    pub status: Option<String>,
    pub attempts: Option<i64>,
    pub error_message: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}
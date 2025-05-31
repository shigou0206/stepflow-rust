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
    pub last_error_at: Option<NaiveDateTime>,
    pub next_retry_at: Option<NaiveDateTime>,

    pub queued_at: NaiveDateTime,
    pub processing_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub failed_at: Option<NaiveDateTime>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Default)]
pub struct UpdateQueueTask {
    pub status: Option<String>,
    pub attempts: Option<i64>,
    pub error_message: Option<String>,
    pub last_error_at: Option<NaiveDateTime>,
    pub next_retry_at: Option<NaiveDateTime>,
    pub processing_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub failed_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowState {
    pub state_id: String,
    pub run_id: String,
    pub shard_id: i64,
    pub state_name: String,
    pub state_type: String,
    pub status: String,
    pub input: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub error_details: Option<String>,
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub version: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateWorkflowState {
    pub state_name: Option<String>,
    pub state_type: Option<String>,
    pub status: Option<String>,
    pub input: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub error_details: Option<String>,
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
    pub version: Option<i64>,
}

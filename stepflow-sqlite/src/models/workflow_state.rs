use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
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

impl Default for WorkflowState {
    fn default() -> Self {
        Self {
            state_id: String::new(),
            run_id: String::new(),
            shard_id: 0,
            state_name: String::new(),
            state_type: String::new(),
            status: String::new(),
            input: None,
            output: None,
            error: None,
            error_details: None,
            started_at: None,
            completed_at: None,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
            version: 0,
        }
    }
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

use chrono::NaiveDateTime;   
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowExecution {
    pub run_id: String,
    pub workflow_id: Option<String>,
    pub shard_id: i64,
    pub template_id: Option<String>,
    pub mode: String,
    pub current_state_name: Option<String>,
    pub status: String,
    pub workflow_type: String,
    pub input: Option<String>,
    pub input_version: i64,
    pub result: Option<String>,
    pub result_version: i64,
    pub start_time: NaiveDateTime,
    pub close_time: Option<NaiveDateTime>,
    pub current_event_id: i64,
    pub memo: Option<String>,
    pub search_attrs: Option<String>,
    pub context_snapshot: Option<String>,
    pub version: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateWorkflowExecution {
    pub workflow_id: Option<String>,
    pub shard_id: Option<i64>,
    pub template_id: Option<String>,
    pub mode: Option<String>,
    pub current_state_name: Option<String>,
    pub status: Option<String>,
    pub workflow_type: Option<String>,
    pub input: Option<String>,
    pub input_version: Option<i64>,
    pub result: Option<String>,
    pub result_version: Option<i64>,
    pub start_time: Option<NaiveDateTime>,
    pub close_time: Option<NaiveDateTime>,
    pub current_event_id: Option<i64>,
    pub memo: Option<String>,
    pub search_attrs: Option<String>,
    pub context_snapshot: Option<String>,
    pub version: Option<i64>,
}
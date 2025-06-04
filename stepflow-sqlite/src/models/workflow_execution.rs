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

impl Default for WorkflowExecution {
    fn default() -> Self {
        Self {
            run_id: String::new(),
            workflow_id: None,
            shard_id: 0,
            template_id: None,
            mode: String::new(),
            current_state_name: None,
            status: String::new(),
            workflow_type: String::new(),
            input: None,
            input_version: 0,
            result: None,
            result_version: 0,
            start_time: chrono::NaiveDateTime::default(),
            close_time: None,
            current_event_id: 0,
            memo: None,
            search_attrs: None,
            context_snapshot: None,
            version: 0,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateWorkflowExecution {
    pub workflow_id: Option<Option<String>>,
    pub shard_id: Option<i64>,
    pub template_id: Option<Option<String>>,
    pub mode: Option<String>,
    pub current_state_name: Option<Option<String>>,
    pub status: Option<String>,
    pub workflow_type: Option<String>,
    pub input: Option<Option<String>>,
    pub input_version: Option<i64>,
    pub result: Option<Option<String>>,
    pub result_version: Option<i64>,
    pub start_time: Option<NaiveDateTime>,
    pub close_time: Option<Option<NaiveDateTime>>,
    pub current_event_id: Option<i64>,
    pub memo: Option<Option<String>>,
    pub search_attrs: Option<Option<String>>,
    pub context_snapshot: Option<Option<String>>,
    pub version: Option<i64>,
}
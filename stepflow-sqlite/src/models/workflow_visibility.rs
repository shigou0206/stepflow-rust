use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowVisibility {
    pub run_id: String,
    pub workflow_id: Option<String>,
    pub workflow_type: Option<String>,
    pub start_time: Option<NaiveDateTime>,
    pub close_time: Option<NaiveDateTime>,
    pub status: Option<String>,
    pub memo: Option<String>,
    pub search_attrs: Option<String>,
    pub version: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateWorkflowVisibility {
    pub workflow_id: Option<Option<String>>,
    pub workflow_type: Option<Option<String>>,
    pub start_time: Option<Option<NaiveDateTime>>,
    pub close_time: Option<Option<NaiveDateTime>>,
    pub status: Option<Option<String>>,
    pub memo: Option<Option<String>>,
    pub search_attrs: Option<Option<String>>,
    pub version: Option<i64>,
}
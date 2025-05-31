use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowEvent {
    pub id: i64,
    pub run_id: String,
    pub shard_id: i64,
    pub event_id: i64,
    pub event_type: String,
    pub state_id: Option<String>,
    pub state_type: Option<String>,
    pub trace_id: Option<String>,
    pub parent_event_id: Option<i64>,
    pub context_version: Option<i64>,
    pub attributes: Option<String>,
    pub attr_version: i64,
    pub timestamp: NaiveDateTime,
    pub archived: bool,
}


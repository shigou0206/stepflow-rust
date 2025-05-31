use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
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

impl Default for WorkflowEvent {
    fn default() -> Self {
        Self {
            id: 0,
            run_id: String::new(),
            shard_id: 0,
            event_id: 0,
            event_type: String::new(),
            state_id: None,
            state_type: None,
            trace_id: None,
            parent_event_id: None,
            context_version: None,
            attributes: None,
            attr_version: 0,
            timestamp: chrono::NaiveDateTime::default(),
            archived: false,
        }
    }
}


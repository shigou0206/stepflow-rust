use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredWorkflowEvent {
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

// Typically, events are immutable, so an UpdateStoredWorkflowEvent might not be needed.
// If archival or soft delete is implemented, it might be handled by a status field or a separate table.

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateStoredWorkflowEvent {
    pub event_type: Option<String>,
    pub state_id: Option<Option<String>>,
    pub state_type: Option<Option<String>>,
    pub trace_id: Option<Option<String>>,
    pub parent_event_id: Option<Option<i64>>,
    pub context_version: Option<Option<i64>>,
    pub attributes: Option<Option<String>>,
    pub attr_version: Option<i64>,
    pub timestamp: Option<NaiveDateTime>,
    pub archived: Option<bool>,
} 
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::dto::engine_event::EngineEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub run_id: String,
    pub event: EngineEvent,
}
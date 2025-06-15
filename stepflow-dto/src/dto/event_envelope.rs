use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::dto::engine_event::EngineEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventEnvelope {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub payload: EngineEvent,
}

impl From<EngineEvent> for EventEnvelope {
    fn from(event: EngineEvent) -> Self {
        EventEnvelope {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            source: "engine".into(),
            payload: event,
        }
    }
}
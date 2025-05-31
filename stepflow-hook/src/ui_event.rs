use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiEvent {
    pub event_type: String,
    pub run_id: String,
    pub state_name: Option<String>,
    pub payload: Value,
}
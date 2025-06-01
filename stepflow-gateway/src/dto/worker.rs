use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct PollRequest {
    pub worker_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PollResponse {
    pub has_task: bool,
    pub run_id: Option<String>,
    pub state_name: Option<String>,
    pub input: Option<Value>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRequest {
    pub run_id: String,
    pub state_name: String,
    pub status: String,
    pub result: Value,
}

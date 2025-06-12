use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrbStartExecutionRequest {
    pub mode: String,
    pub template_id: Option<String>,
    pub dsl: Option<Value>,
    pub init_ctx: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrbExecUpdateRequest {
    pub status: String,
    pub result: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrbListRequest {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrbListByStatusRequest {
    pub status: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrbExecutionResult {
    pub run_id: String,
    pub mode: String,
    pub status: String,
    pub result: Option<Value>,
    pub started_at: String,
    pub finished_at: Option<String>,
}

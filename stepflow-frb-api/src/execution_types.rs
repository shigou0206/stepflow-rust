use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrbStartExecutionRequest {
    pub mode: String,
    pub template_id: Option<String>,
    pub dsl_json: Option<String>,
    pub init_ctx_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrbExecUpdateRequest {
    pub status: String,
    pub result_json: Option<String>,
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
    pub result_json: Option<String>,
    pub started_at: String,
    pub finished_at: Option<String>,
}
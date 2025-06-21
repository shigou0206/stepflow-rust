use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 工作流运行状态信息（用于 get_status 返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineStatusDto {
    pub run_id: String,
    pub current_state: String,
    pub last_task_state: Option<String>,
    pub status: String, // RUNNING / COMPLETED / FAILED ...
    pub parent_run_id: Option<String>,
    pub parent_state_name: Option<String>,
    pub context: Value, // 当前上下文
    pub updated_at: DateTime<Utc>,
    pub finished: bool,
}

/// 控制类请求：cancel / terminate / pause / resume / cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlRequest {
    pub run_id: String,
}

/// 重试请求：可选指定 state_name
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryRequest {
    pub run_id: String,
    pub state_name: Option<String>,
}
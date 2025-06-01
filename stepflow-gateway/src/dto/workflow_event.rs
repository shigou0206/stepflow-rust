use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use utoipa::ToSchema;

/// 工作流事件记录请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct RecordEventRequest {
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
    #[serde(default)]
    pub archived: bool,
}

/// 工作流事件 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct WorkflowEventDto {
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

/// 分页查询参数
#[derive(Debug, Deserialize, ToSchema)]
pub struct ListQuery {
    /// 每页数量，默认 100
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// 偏移量，默认 0
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    100
} 
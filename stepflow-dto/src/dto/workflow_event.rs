use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use utoipa::{ToSchema, IntoParams};
use stepflow_storage::entities::workflow_event::StoredWorkflowEvent;
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
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
} 

impl From<StoredWorkflowEvent> for WorkflowEventDto {
    fn from(event: StoredWorkflowEvent) -> Self {
        Self {
            id: event.id,
            run_id: event.run_id,
            shard_id: event.shard_id,
            event_id: event.event_id,
            event_type: event.event_type,
            state_id: event.state_id,
            state_type: event.state_type,
            trace_id: event.trace_id,
            parent_event_id: event.parent_event_id,
            context_version: event.context_version,
            attributes: event.attributes.and_then(|s| serde_json::from_str(&s).ok()),
            attr_version: event.attr_version,
            timestamp: event.timestamp,
            archived: event.archived,
        }
    }
}
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::Value;
use utoipa::{ToSchema, IntoParams};

/// 分页查询参数
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
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

/// Activity Task 的响应 DTO
#[derive(Debug, Serialize, ToSchema)]
pub struct ActivityTaskDto {
    pub task_token: String,
    pub run_id: String,
    pub activity_type: String,
    pub status: String,
    pub input: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub error_details: Option<String>,
    pub attempt: i64,
    pub max_attempts: i64,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub heartbeat_at: Option<DateTime<Utc>>,
}

/// 完成任务的请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct CompleteRequest {
    pub result: Value,
}

/// 任务失败的请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct FailRequest {
    pub reason: String,
    pub details: Option<String>,
}

/// 心跳请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct HeartbeatRequest {
    pub details: Option<String>,
} 
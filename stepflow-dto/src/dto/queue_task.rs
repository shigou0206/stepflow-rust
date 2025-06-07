use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct QueueTaskDto {
    pub task_id: String,                        // 唯一任务ID
    pub run_id: String,                         // 工作流运行ID
    pub state_name: String,                     // 节点状态名
    pub resource: String,                       // 资源类型（工具标识，如 "http"）
    pub task_payload: Option<Value>,            // 上下文数据（通常为输入）
    pub status: String,                         // 状态（pending, processing, completed 等）
    pub attempts: i64,                          // 当前重试次数
    pub max_attempts: i64,                      // 最大重试次数
    pub priority: Option<u8>,                   // 优先级（0-255，越大越高）
    pub timeout_seconds: Option<i64>,           // 超时时间（秒）
    pub error_message: Option<String>,          // 错误信息（如有）
    pub last_error_at: Option<DateTime<Utc>>,   // 上次错误时间
    pub next_retry_at: Option<DateTime<Utc>>,   // 下一次重试时间
    pub queued_at: DateTime<Utc>,               // 入队时间
    pub processing_at: Option<DateTime<Utc>>,   // 开始处理时间
    pub completed_at: Option<DateTime<Utc>>,    // 完成时间
    pub failed_at: Option<DateTime<Utc>>,       // 失败时间
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema)]
pub struct UpdateQueueTaskDto {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_payload: Option<Option<Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attempts: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<Option<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error_at: Option<Option<DateTime<Utc>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_retry_at: Option<Option<DateTime<Utc>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_at: Option<Option<DateTime<Utc>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<Option<DateTime<Utc>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<Option<DateTime<Utc>>>,
}
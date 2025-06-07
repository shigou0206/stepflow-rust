use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::Value;
use utoipa::{ToSchema, IntoParams};
use stepflow_storage::entities::activity_task::StoredActivityTask;
/// 活动任务列表查询参数
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    /// 每页数量
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// 偏移量
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 { 20 }

/// 活动任务详情
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActivityTaskDto {
    /// 任务令牌
    pub task_token: String,
    /// 工作流运行ID
    pub run_id: String,
    /// 活动类型
    pub activity_type: String,
    /// 任务状态
    pub status: String,
    /// 输入数据
    pub input: Option<Value>,
    /// 执行结果
    pub result: Option<Value>,
    /// 错误原因
    pub error: Option<String>,
    /// 错误详情
    pub error_details: Option<String>,
    /// 当前重试次数
    pub attempt: i64,
    /// 最大重试次数
    pub max_attempts: i64,
    /// 调度时间
    pub scheduled_at: DateTime<Utc>,
    /// 开始时间
    pub started_at: Option<DateTime<Utc>>,
    /// 完成时间
    pub completed_at: Option<DateTime<Utc>>,
    /// 最近心跳时间
    pub heartbeat_at: Option<DateTime<Utc>>,
}

/// 完成任务请求
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CompleteRequest {
    /// 执行结果
    pub result: Value,
}

/// 标记任务失败请求
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FailRequest {
    /// 失败原因
    pub reason: String,
    /// 错误详情
    pub details: Option<String>,
}

/// 任务心跳请求
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatRequest {
    /// 心跳详情
    pub _details: Option<String>,
} 

impl From<StoredActivityTask> for ActivityTaskDto {
    fn from(task: StoredActivityTask) -> Self {
        Self {
            task_token: task.task_token,
            run_id: task.run_id,
            activity_type: task.activity_type,
            status: task.status,
            input: task.input,
            result: task.result,
            error: task.error,
            error_details: task.error_details,
            attempt: task.attempt,
            max_attempts: task.max_attempts,
            scheduled_at: task.scheduled_at.and_utc(),
            started_at: task.started_at.map(|dt| dt.and_utc()),
            completed_at: task.completed_at.map(|dt| dt.and_utc()),
            heartbeat_at: task.heartbeat_at.map(|dt| dt.and_utc()),
        }
    }
}

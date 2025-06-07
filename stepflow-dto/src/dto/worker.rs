use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

/// Worker 请求任务的结构
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PollRequest {
    /// 唯一 Worker 标识
    pub worker_id: String,
    /// 支持的工具类型，例如 ["http", "shell"]
    pub capabilities: Vec<String>,
}

/// Worker 请求任务响应结构（精简版）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PollResponse {
    /// 是否成功获取到任务
    pub has_task: bool,

    /// 工作流运行 ID（如有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,

    /// 节点状态名称（如有）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_name: Option<String>,

    /// 工具类型（如 "http"、"shell"，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_type: Option<String>,

    /// 工具执行所需的输入参数（上下文片段）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<Value>,

    /// 任务 ID（可用于追踪或重试，后续扩展）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
}

/// Worker 上报状态的枚举（执行结果）
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    /// 执行成功
    SUCCEEDED,
    /// 执行失败
    FAILED,
}

/// Worker 上报任务状态更新的结构
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateRequest {
    /// 工作流运行 ID
    pub run_id: String,

    /// 节点状态名称
    pub state_name: String,

    /// 执行结果状态
    pub status: TaskStatus,

    /// 工具执行输出（可为成功数据或错误详情）
    pub result: Value,

    /// 可选：任务耗时（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,

    /// 可选：任务 ID（如支持任务级别跟踪）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
}
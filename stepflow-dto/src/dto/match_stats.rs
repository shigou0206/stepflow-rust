use serde::{Deserialize, Serialize};
use crate::dto::queue_task::QueueTaskDto;
use utoipa::{ToSchema};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EnqueueRequest {
    #[schema(example = "default_task_queue")]
    pub queue: String,
    pub task: QueueTaskDto,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PollRequest {
    #[schema(example = "default_task_queue")]
    pub queue: String,
    #[schema(example = "worker-1")]
    pub worker_id: String,
    #[schema(example = 10)]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PollResponse {
    pub has_task: bool,
    pub task: Option<QueueTaskDto>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MatchStats {
    pub queue: String,
    pub pending_tasks: usize,
    pub waiting_workers: usize,
}
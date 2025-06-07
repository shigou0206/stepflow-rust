use async_trait::async_trait;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::NaiveDateTime;
use std::sync::Arc;
use std::any::Any;
use stepflow_storage::persistence_manager::PersistenceManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchStats {
    pub queue: String,
    pub pending_tasks: usize,
    pub waiting_workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub run_id: String,                      // 工作流实例ID
    pub state_name: String,                  // 当前任务对应的状态名
    pub input: Option<Value>,                // 任务输入
    pub task_type: String,                   // 任务类型
    pub task_token: Option<String>,          // 任务的唯一标识符
    pub priority: Option<u8>,                // 任务优先级 (0-255, 越大优先级越高)
    pub attempt: Option<i64>,                // 当前尝试次数
    pub max_attempts: Option<i64>,           // 最大允许的尝试次数
    pub timeout_seconds: Option<i64>,        // 任务执行的超时时间
    pub scheduled_at: Option<NaiveDateTime>, // 任务预定的执行时间
}

impl Default for Task {
    fn default() -> Self {
        Self {
            run_id: String::new(),
            state_name: String::new(),
            input: None,
            task_type: "default".to_string(),
            task_token: None,
            priority: Some(128), // 默认中等优先级
            attempt: Some(0),
            max_attempts: Some(3), // 默认最多重试3次
            timeout_seconds: Some(300), // 默认5分钟超时
            scheduled_at: None,
        }
    }
}

impl Task {
    pub fn new(run_id: String, state_name: String) -> Self {
        Self {
            run_id,
            state_name,
            ..Default::default()
        }
    }

    pub fn with_input(mut self, input: Value) -> Self {
        self.input = Some(input);
        self
    }

    pub fn with_type(mut self, task_type: String) -> Self {
        self.task_type = task_type;
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn with_timeout(mut self, timeout_seconds: i64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }
}

#[async_trait]
pub trait MatchService: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    async fn queue_stats(&self) -> Vec<MatchStats> {
        vec![]
    }

    async fn poll_task(&self, queue: &str, worker_id: &str, timeout: Duration) -> Option<Task>;
    async fn enqueue_task(&self, queue: &str, task: Task) -> Result<(), String>;
    async fn wait_for_completion(
        &self,
        run_id: &str,
        state_name: &str,
        input: &Value,
        persistence: Arc<dyn PersistenceManager>,
    ) -> Result<Value, String>;
}
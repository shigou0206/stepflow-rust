pub mod template;

use crate::error::AppResult;
use async_trait::async_trait;
use crate::dto::template::*;
use serde_json::Value;
use chrono::{DateTime, Utc};
#[async_trait]
pub trait TemplateService: Clone + Send + Sync + 'static {
    async fn create(&self, dto: TemplateUpsert)               -> AppResult<TemplateDto>;
    async fn update(&self, id: &str, dto: TemplateUpsert)     -> AppResult<TemplateDto>;
    async fn get   (&self, id: &str)                          -> AppResult<TemplateDto>;
    async fn list  (&self)                                    -> AppResult<Vec<TemplateDto>>;
    async fn delete(&self, id: &str)                          -> AppResult<()>;
}

pub use template::TemplateSqlxSvc as TemplateSvc;

// gateway/src/service/mod.rs
pub mod execution;
pub use execution::ExecutionSqlxSvc as ExecutionSvc;
use crate::dto::execution::*;

#[async_trait]
pub trait ExecutionService: Clone + Send + Sync + 'static {
    async fn start(&self, req: ExecStart) -> AppResult<ExecDto>;
    async fn get  (&self, run_id: &str) -> AppResult<ExecDto>;
    async fn list (&self, limit: i64, offset: i64) -> AppResult<Vec<ExecDto>>;
    async fn update(&self, run_id: &str, status: String, result: Option<Value>) -> AppResult<()>;
    async fn delete(&self, run_id: &str) -> AppResult<()>;
    async fn list_by_status(&self, status: &str, limit: i64, offset: i64) -> AppResult<Vec<ExecDto>>;
}

pub mod activity_task;
pub use activity_task::ActivityTaskSqlxSvc as ActivityTaskSvc;
use crate::dto::activity_task::*;

#[async_trait]
pub trait ActivityTaskService: Clone + Send + Sync + 'static {
    async fn list_tasks(&self, limit: i64, offset: i64) -> AppResult<Vec<ActivityTaskDto>>;
    async fn get_task(&self, task_token: &str) -> AppResult<ActivityTaskDto>;
    async fn get_tasks_by_run_id(&self, run_id: &str) -> AppResult<Vec<ActivityTaskDto>>;
    async fn start_task(&self, task_token: &str) -> AppResult<ActivityTaskDto>;
    async fn complete_task(&self, task_token: &str, result: Value) -> AppResult<ActivityTaskDto>;
    async fn fail_task(&self, task_token: &str, req: FailRequest) -> AppResult<ActivityTaskDto>;
    async fn heartbeat_task(&self, task_token: &str, req: HeartbeatRequest) -> AppResult<ActivityTaskDto>;
}

pub mod workflow_event;
pub use workflow_event::WorkflowEventSqlxSvc as WorkflowEventSvc;

pub mod queue_task;
pub use queue_task::QueueTaskSqlxSvc as QueueTaskSvc;
use crate::dto::queue_task::*;

#[async_trait]
pub trait QueueTaskService: Send + Sync {
    async fn get_task(&self, task_id: &str) -> AppResult<QueueTaskDto>;
    async fn list_tasks_by_status(&self, status: &str, limit: i64, offset: i64) -> AppResult<Vec<QueueTaskDto>>;
    async fn update_task(&self, task_id: &str, update: UpdateQueueTaskDto) -> AppResult<()>;
}

pub mod timer;
pub use timer::TimerSqlxSvc as TimerSvc;
use crate::dto::timer::*;

#[async_trait]
#[async_trait]
pub trait TimerService: Send + Sync {
    async fn create_timer(&self, dto: CreateTimerDto) -> AppResult<TimerDto>;
    async fn get_timer(&self, timer_id: &str) -> AppResult<TimerDto>;
    async fn update_timer(&self, timer_id: &str, update: UpdateTimerDto) -> AppResult<TimerDto>;
    async fn delete_timer(&self, timer_id: &str) -> AppResult<()>;
    async fn find_timers_before(&self, before: DateTime<Utc>, limit: i64) -> AppResult<Vec<TimerDto>>;
}


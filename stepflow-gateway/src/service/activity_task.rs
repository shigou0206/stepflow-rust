use async_trait::async_trait;
use chrono::{DateTime, Utc};
use stepflow_storage::PersistenceManager;
use stepflow_sqlite::models::activity_task::{ActivityTask, UpdateActivityTask};
use crate::dto::activity_task::*;
use crate::error::{AppResult, AppError};
use crate::app_state::AppState;
use std::sync::Arc;
use serde_json::Value;

#[async_trait]
pub trait ActivityTaskService: Clone + Send + Sync + 'static {
    async fn get_task(&self, task_token: &str) -> AppResult<ActivityTaskDto>;
    async fn get_tasks_by_run_id(&self, run_id: &str) -> AppResult<Vec<ActivityTaskDto>>;
    async fn list_tasks(&self, limit: i64, offset: i64) -> AppResult<Vec<ActivityTaskDto>>;
    async fn start_task(&self, task_token: &str) -> AppResult<ActivityTaskDto>;
    async fn complete_task(&self, task_token: &str, result: Value) -> AppResult<ActivityTaskDto>;
    async fn fail_task(&self, task_token: &str, reason: String, details: Option<String>) -> AppResult<ActivityTaskDto>;
    async fn heartbeat_task(&self, task_token: &str, details: Option<String>) -> AppResult<ActivityTaskDto>;
}

#[derive(Clone)]
pub struct ActivityTaskSvc {
    persist: Arc<dyn PersistenceManager>,
}

impl ActivityTaskSvc {
    pub fn new(persist: Arc<dyn PersistenceManager>) -> Self {
        Self { persist }
    }

    fn to_dto(task: ActivityTask) -> ActivityTaskDto {
        ActivityTaskDto {
            task_token: task.task_token,
            run_id: task.run_id,
            activity_type: task.activity_type,
            status: task.status,
            input: task.input.and_then(|s| serde_json::from_str(&s).ok()),
            result: task.result.and_then(|s| serde_json::from_str(&s).ok()),
            error: task.error,
            error_details: task.error_details,
            attempt: task.attempt,
            max_attempts: task.max_attempts,
            scheduled_at: DateTime::from_utc(task.scheduled_at, Utc),
            started_at: task.started_at.map(|t| DateTime::from_utc(t, Utc)),
            completed_at: task.completed_at.map(|t| DateTime::from_utc(t, Utc)),
            heartbeat_at: task.heartbeat_at.map(|t| DateTime::from_utc(t, Utc)),
        }
    }
}

#[async_trait]
impl ActivityTaskService for ActivityTaskSvc {
    /// 获取单个任务
    async fn get_task(&self, task_token: &str) -> AppResult<ActivityTaskDto> {
        let task = self.persist.get_task(task_token).await?
            .ok_or(AppError::NotFound)?;
        Ok(Self::to_dto(task))
    }

    /// 获取工作流实例的所有任务
    async fn get_tasks_by_run_id(&self, run_id: &str) -> AppResult<Vec<ActivityTaskDto>> {
        let tasks = self.persist.find_tasks_by_status("running", 100, 0).await?;
        let tasks = tasks.into_iter()
            .filter(|t| t.run_id == run_id)
            .collect::<Vec<_>>();
        Ok(tasks.into_iter().map(Self::to_dto).collect())
    }

    /// 分页获取任务列表
    async fn list_tasks(&self, limit: i64, offset: i64) -> AppResult<Vec<ActivityTaskDto>> {
        let tasks = self.persist.find_tasks_by_status("", limit, offset).await?;
        Ok(tasks.into_iter().map(Self::to_dto).collect())
    }

    /// 开始执行任务
    async fn start_task(&self, task_token: &str) -> AppResult<ActivityTaskDto> {
        let task = self.persist.get_task(task_token).await?
            .ok_or(AppError::NotFound)?;

        if task.status != "scheduled" {
            return Err(AppError::BadRequest("Task cannot be started".into()));
        }

        let changes = UpdateActivityTask {
            status: Some("running".to_string()),
            started_at: Some(Utc::now().naive_utc()),
            attempt: Some(task.attempt + 1),
            ..Default::default()
        };

        self.persist.update_task(task_token, &changes).await?;
        self.get_task(task_token).await
    }

    /// 完成任务
    async fn complete_task(&self, task_token: &str, result: Value) -> AppResult<ActivityTaskDto> {
        let task = self.persist.get_task(task_token).await?
            .ok_or(AppError::NotFound)?;

        if task.status != "running" {
            return Err(AppError::BadRequest("Task is not running".into()));
        }

        let changes = UpdateActivityTask {
            status: Some("completed".to_string()),
            result: Some(result.to_string()),
            completed_at: Some(Utc::now().naive_utc()),
            ..Default::default()
        };

        self.persist.update_task(task_token, &changes).await?;
        self.get_task(task_token).await
    }

    /// 标记任务失败
    async fn fail_task(&self, task_token: &str, reason: String, details: Option<String>) -> AppResult<ActivityTaskDto> {
        let task = self.persist.get_task(task_token).await?
            .ok_or(AppError::NotFound)?;

        if task.status != "running" {
            return Err(AppError::BadRequest("Task is not running".into()));
        }

        let changes = UpdateActivityTask {
            status: Some("failed".to_string()),
            error: Some(reason),
            error_details: details,
            completed_at: Some(Utc::now().naive_utc()),
            ..Default::default()
        };

        self.persist.update_task(task_token, &changes).await?;
        self.get_task(task_token).await
    }

    /// 更新任务心跳
    async fn heartbeat_task(&self, task_token: &str, details: Option<String>) -> AppResult<ActivityTaskDto> {
        let task = self.persist.get_task(task_token).await?
            .ok_or(AppError::NotFound)?;

        if task.status != "running" {
            return Err(AppError::BadRequest("Task is not running".into()));
        }

        let changes = UpdateActivityTask {
            heartbeat_at: Some(Utc::now().naive_utc()),
            ..Default::default()
        };

        self.persist.update_task(task_token, &changes).await?;
        self.get_task(task_token).await
    }
} 
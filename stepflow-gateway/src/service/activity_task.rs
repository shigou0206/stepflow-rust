use async_trait::async_trait;
use stepflow_storage::persistence_manager::PersistenceManager;   
use stepflow_storage::error::StorageError;
use stepflow_storage::entities::activity_task::{StoredActivityTask, UpdateStoredActivityTask};
use anyhow::Error;
use crate::{
    dto::activity_task::*,
    error::{AppResult, AppError},
    service::ActivityTaskService,
};
use serde_json::Value;
use stepflow_sqlite::models::activity_task::ActivityTask;

#[derive(Clone)]
pub struct ActivityTaskSqlxSvc {
    pm: std::sync::Arc<dyn PersistenceManager>,
}

impl ActivityTaskSqlxSvc {
    pub fn new(pm: std::sync::Arc<dyn PersistenceManager>) -> Self {
        Self { pm }
    }
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

impl From<ActivityTask> for ActivityTaskDto {
    fn from(task: ActivityTask) -> Self {
        Self {
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
            scheduled_at: task.scheduled_at.and_utc(),
            started_at: task.started_at.map(|dt| dt.and_utc()),
            completed_at: task.completed_at.map(|dt| dt.and_utc()),
            heartbeat_at: task.heartbeat_at.map(|dt| dt.and_utc()),
        }
    }
}

#[async_trait]
impl ActivityTaskService for ActivityTaskSqlxSvc {
    async fn list_tasks(&self, limit: i64, offset: i64) -> AppResult<Vec<ActivityTaskDto>> {
        let tasks = self.pm.find_tasks_by_status("SCHEDULED", limit, offset).await
            .map_err(|e: StorageError| Error::new(e))?;
        Ok(tasks.into_iter().map(|task| task.into()).collect())
    }

    async fn get_task(&self, task_token: &str) -> AppResult<ActivityTaskDto> {
        let task = self.pm.get_task(task_token).await
            .map_err(|e: StorageError| Error::new(e))?
            .ok_or(AppError::NotFound)?;
        Ok(task.into())
    }

    async fn get_tasks_by_run_id(&self, run_id: &str) -> AppResult<Vec<ActivityTaskDto>> {
        let tasks = self.pm.find_tasks_by_status("RUNNING", 100, 0).await
            .map_err(|e: StorageError| Error::new(e))?
            .into_iter()
            .filter(|task| task.run_id == run_id)
            .collect::<Vec<_>>();
        Ok(tasks.into_iter().map(|task| task.into()).collect())
    }

    async fn start_task(&self, task_token: &str) -> AppResult<ActivityTaskDto> {
        let task = self.pm.get_task(task_token).await
            .map_err(|e: StorageError| Error::new(e))?
            .ok_or(AppError::NotFound)?;
        
        if task.status != "SCHEDULED" {
            return Err(AppError::BadRequest(format!(
                "Activity task {} is not in scheduled state: {}",
                task_token, task.status
            )));
        }

        let changes = UpdateStoredActivityTask {
            status: Some("RUNNING".to_string()),
            started_at: Some(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        self.pm.update_task(task_token, &changes).await
            .map_err(|e: StorageError| Error::new(e))?;

        let task = self.pm.get_task(task_token).await
            .map_err(|e: StorageError| Error::new(e))?
            .ok_or(AppError::NotFound)?;
        Ok(task.into())
    }

    async fn complete_task(&self, task_token: &str, result: Value) -> AppResult<ActivityTaskDto> {
        let task = self.pm.get_task(task_token).await
            .map_err(|e: StorageError| Error::new(e))?
            .ok_or(AppError::NotFound)?;
        
        if task.status != "RUNNING" {
            return Err(AppError::BadRequest(format!(
                "Activity task {} is not in running state: {}",
                task_token, task.status
            )));
        }

        let changes = UpdateStoredActivityTask {
            status: Some("COMPLETED".to_string()),
            result: Some(Some(result)),
            completed_at: Some(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        self.pm.update_task(task_token, &changes).await
            .map_err(|e: StorageError| Error::new(e))?;

        let task = self.pm.get_task(task_token).await
            .map_err(|e: StorageError| Error::new(e))?
            .ok_or(AppError::NotFound)?;
        Ok(task.into())
    }

    async fn fail_task(&self, task_token: &str, req: FailRequest) -> AppResult<ActivityTaskDto> {
        let task = self.pm.get_task(task_token).await
            .map_err(|e: StorageError| Error::new(e))?
            .ok_or(AppError::NotFound)?;
        
        if task.status != "RUNNING" {
            return Err(AppError::BadRequest(format!(
                "Activity task {} is not in running state: {}",
                task_token, task.status
            )));
        }

        let changes = UpdateStoredActivityTask {
            status: Some("FAILED".to_string()),
            error: Some(Some(req.reason)),
            error_details: Some(req.details),
            completed_at: Some(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        self.pm.update_task(task_token, &changes).await
            .map_err(|e: StorageError| Error::new(e))?;

        let task = self.pm.get_task(task_token).await
            .map_err(|e: StorageError| Error::new(e))?
            .ok_or(AppError::NotFound)?;
        Ok(task.into())
    }

    async fn heartbeat_task(&self, task_token: &str, _req: HeartbeatRequest) -> AppResult<ActivityTaskDto> {
        let task = self.pm.get_task(task_token).await
            .map_err(|e: StorageError| Error::new(e))?
            .ok_or(AppError::NotFound)?;
        
        if task.status != "RUNNING" {
            return Err(AppError::BadRequest(format!(
                "Activity task {} is not in running state: {}",
                task_token, task.status
            )));
        }

        let changes = UpdateStoredActivityTask {
            heartbeat_at: Some(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        self.pm.update_task(task_token, &changes).await
            .map_err(|e: StorageError| Error::new(e))?;

        let task = self.pm.get_task(task_token).await
            .map_err(|e: StorageError| Error::new(e))?
            .ok_or(AppError::NotFound)?;
        Ok(task.into())
    }
}
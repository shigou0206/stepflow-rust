//! activity_task_sqlx_svc.rs
//! —— 依赖 sqlx 后端的 ActivityTaskService 实现（使用统一 DynPM 别名）

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;

use stepflow_dto::dto::activity_task::*;
use stepflow_storage::db::DynPM;
use stepflow_storage::{
    entities::activity_task::UpdateStoredActivityTask,
    error::StorageError,
};

use crate::service::ActivityTaskService;

use stepflow_core::error::{AppError, AppResult};

/// ------------------------------------------------------------
/// 具体实现结构体
/// ------------------------------------------------------------
#[derive(Clone)]
pub struct ActivityTaskSqlxSvc {
    pm: DynPM,
}

impl ActivityTaskSqlxSvc {
    pub fn new(pm: DynPM) -> Self {
        Self { pm }
    }
}

/// ------------------------------------------------------------
/// ActivityTaskService trait 实现
/// ------------------------------------------------------------
#[async_trait]
impl ActivityTaskService for ActivityTaskSqlxSvc {
    async fn list_tasks(&self, limit: i64, offset: i64) -> AppResult<Vec<ActivityTaskDto>> {
        let tasks = self
            .pm
            .find_tasks_by_status("SCHEDULED", limit, offset)
            .await
            .map_err(AppError::from_storage)?;          // ↩️ 统一错误包装
        Ok(tasks.into_iter().map(Into::into).collect())
    }

    async fn get_task(&self, task_token: &str) -> AppResult<ActivityTaskDto> {
        let task = self
            .pm
            .get_task(task_token)
            .await
            .map_err(AppError::from_storage)?
            .ok_or(AppError::NotFound)?;
        Ok(task.into())
    }

    async fn get_tasks_by_run_id(&self, run_id: &str) -> AppResult<Vec<ActivityTaskDto>> {
        let tasks = self
            .pm
            .find_tasks_by_status("RUNNING", 100, 0)
            .await
            .map_err(AppError::from_storage)?
            .into_iter()
            .filter(|t| t.run_id == run_id)
            .map(Into::into)
            .collect();
        Ok(tasks)
    }

    async fn start_task(&self, task_token: &str) -> AppResult<ActivityTaskDto> {
        let task = self.get_task(task_token).await?; // 复用 get_task 做存在检查

        if task.status != "SCHEDULED" {
            return Err(AppError::BadRequest(format!(
                "Activity task {} is not in scheduled state: {}",
                task_token, task.status
            )));
        }

        self.pm
            .update_task(
                task_token,
                &UpdateStoredActivityTask {
                    status:      Some("RUNNING".into()),
                    started_at:  Some(Some(Utc::now().naive_utc())),
                    ..Default::default()
                },
            )
            .await
            .map_err(AppError::from_storage)?;

        self.get_task(task_token).await
    }

    async fn complete_task(&self, task_token: &str, result: Value) -> AppResult<ActivityTaskDto> {
        let task = self.get_task(task_token).await?;
        if task.status != "RUNNING" {
            return Err(AppError::BadRequest(format!(
                "Activity task {} is not in running state: {}",
                task_token, task.status
            )));
        }

        self.pm
            .update_task(
                task_token,
                &UpdateStoredActivityTask {
                    status:       Some("COMPLETED".into()),
                    result:       Some(Some(result)),
                    completed_at: Some(Some(Utc::now().naive_utc())),
                    ..Default::default()
                },
            )
            .await
            .map_err(AppError::from_storage)?;

        self.get_task(task_token).await
    }

    async fn fail_task(&self, task_token: &str, req: FailRequest) -> AppResult<ActivityTaskDto> {
        let task = self.get_task(task_token).await?;
        if task.status != "RUNNING" {
            return Err(AppError::BadRequest(format!(
                "Activity task {} is not in running state: {}",
                task_token, task.status
            )));
        }

        self.pm
            .update_task(
                task_token,
                &UpdateStoredActivityTask {
                    status:        Some("FAILED".into()),
                    error:         Some(Some(req.reason)),
                    error_details: Some(req.details),
                    completed_at:  Some(Some(Utc::now().naive_utc())),
                    ..Default::default()
                },
            )
            .await
            .map_err(AppError::from_storage)?;

        self.get_task(task_token).await
    }

    async fn heartbeat_task(&self, task_token: &str, _req: HeartbeatRequest) -> AppResult<ActivityTaskDto> {
        let task = self.get_task(task_token).await?;
        if task.status != "RUNNING" {
            return Err(AppError::BadRequest(format!(
                "Activity task {} is not in running state: {}",
                task_token, task.status
            )));
        }

        self.pm
            .update_task(
                task_token,
                &UpdateStoredActivityTask {
                    heartbeat_at: Some(Some(Utc::now().naive_utc())),
                    ..Default::default()
                },
            )
            .await
            .map_err(AppError::from_storage)?;

        self.get_task(task_token).await
    }
}

/// ------------------------------------------------------------
/// AppError <-> StorageError 转换辅助
/// ------------------------------------------------------------
trait FromStorageErr<T> {
    fn from_storage(err: StorageError) -> Self;
}

impl FromStorageErr<AppError> for AppError {
    fn from_storage(err: StorageError) -> Self {
        AppError::Internal(err.to_string())
    }
}
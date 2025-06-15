use async_trait::async_trait;
use stepflow_dto::dto::queue_task::{QueueTaskDto, UpdateQueueTaskDto};
use stepflow_core::{
    app_state::AppState,
    error::{AppError, AppResult},
};
use std::sync::Arc;
use stepflow_storage::entities::queue_task::{StoredQueueTask, UpdateStoredQueueTask};
use crate::service::QueueTaskService;
use serde_json::Value;
use stepflow_dsl::WorkflowDSL;
use anyhow::anyhow;

#[derive(Clone)]
pub struct QueueTaskSqlxSvc {
    state: Arc<AppState>,
}

impl QueueTaskSqlxSvc {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    fn to_dto(stored: StoredQueueTask) -> QueueTaskDto {
        QueueTaskDto {
            task_id: stored.task_id,
            run_id: stored.run_id,
            state_name: stored.state_name,
            resource: stored.resource,
            task_payload: stored.task_payload,
            status: stored.status,
            attempts: stored.attempts,
            max_attempts: stored.max_attempts,
            priority: stored.priority.map(|p| p as u8),
            timeout_seconds: stored.timeout_seconds,
            error_message: stored.error_message,
            last_error_at: stored.last_error_at.map(|t| t.and_utc()),
            next_retry_at: stored.next_retry_at.map(|t| t.and_utc()),
            queued_at: stored.queued_at.and_utc(),
            processing_at: stored.processing_at.map(|t| t.and_utc()),
            completed_at: stored.completed_at.map(|t| t.and_utc()),
            failed_at: stored.failed_at.map(|t| t.and_utc()),
        }
    }

    fn to_stored_update(dto: UpdateQueueTaskDto) -> UpdateStoredQueueTask {
        UpdateStoredQueueTask {
            status: dto.status,
            task_payload: dto.task_payload,
            attempts: dto.attempts,
            error_message: dto.error_message,
            priority: dto.priority.map(|p| p as i64),
            timeout_seconds: dto.timeout_seconds,
            last_error_at: dto.last_error_at.map(|opt| opt.map(|dt| dt.naive_utc())),
            next_retry_at: dto.next_retry_at.map(|opt| opt.map(|dt| dt.naive_utc())),
            processing_at: dto.processing_at.map(|opt| opt.map(|dt| dt.naive_utc())),
            completed_at: dto.completed_at.map(|opt| opt.map(|dt| dt.naive_utc())),
            failed_at: dto.failed_at.map(|opt| opt.map(|dt| dt.naive_utc())),
        }
    }

    fn parse_workflow_dsl(val: Value) -> Result<WorkflowDSL, AppError> {
        match val {
            Value::Object(_) => serde_json::from_value(val)
                .map_err(|e| AppError::BadRequest(format!("invalid DSL: {e}"))),
            Value::String(ref s) => serde_json::from_str(s)
                .map_err(|e| AppError::BadRequest(format!("invalid DSL string: {e}"))),
            _ => Err(AppError::BadRequest("DSL must be a JSON object or string".into())),
        }
    }

    pub async fn validate_task_dsl(&self, task_id: &str) -> AppResult<()> {
        let dto = self.get_task(task_id).await?;
        let payload = dto.task_payload.ok_or(AppError::BadRequest("task payload is empty".to_string()))?;
        let _dsl = Self::parse_workflow_dsl(payload)?;
        Ok(())
    }
}

#[async_trait]
impl QueueTaskService for QueueTaskSqlxSvc {
    async fn get_task(&self, task_id: &str) -> AppResult<QueueTaskDto> {
        let stored_opt = self.state.persist.get_queue_task(task_id).await
            .map_err(|e| AppError::Anyhow(anyhow!("get_queue_task failed: {e}")))?;
    
        let stored = stored_opt.ok_or(AppError::NotFound)?;
    
        Ok(Self::to_dto(stored))
    }

    async fn list_tasks_by_status(&self, status: &str, limit: i64, offset: i64) -> AppResult<Vec<QueueTaskDto>> {
        let stored_tasks = self.state.persist.find_queue_tasks_by_status(status, limit, offset).await
            .map_err(|e| AppError::Anyhow(anyhow!("find_queue_tasks_by_status failed: {e}")))?;
        Ok(stored_tasks.into_iter().map(Self::to_dto).collect())
    }

    async fn update_task(&self, task_id: &str, update: UpdateQueueTaskDto) -> AppResult<()> {
        let stored_update = Self::to_stored_update(update);
        self.state.persist.update_queue_task(task_id, &stored_update).await
            .map_err(|e| AppError::Anyhow(anyhow!("update_queue_task failed: {e}")))?;
        Ok(())
    }

    async fn delete_task(&self, task_id: &str) -> AppResult<()> {
        self.state.persist.delete_queue_task(task_id).await
            .map_err(|e| AppError::Anyhow(anyhow!("delete_queue_task failed: {e}")))?;
        Ok(())
    }

    async fn list_tasks_to_retry(&self, before: chrono::NaiveDateTime, limit: i64) -> AppResult<Vec<QueueTaskDto>> {
        let stored_tasks = self.state.persist.find_queue_tasks_to_retry(before, limit).await
            .map_err(|e| AppError::Anyhow(anyhow!("find_queue_tasks_to_retry failed: {e}")))?;
        Ok(stored_tasks.into_iter().map(Self::to_dto).collect())
    }
}
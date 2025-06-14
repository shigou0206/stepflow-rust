use sqlx::SqlitePool;
use crate::{
    crud::queue_task_crud,
    models::queue_task::{QueueTask, UpdateQueueTask},
};
use stepflow_storage::entities::queue_task::{StoredQueueTask, UpdateStoredQueueTask};
use stepflow_storage::error::StorageError;
use chrono::NaiveDateTime;

#[derive(Clone)]
pub struct QueueTaskPersistence {
    pool: SqlitePool,
}

impl QueueTaskPersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // model -> entity
    fn to_entity(model: QueueTask) -> StoredQueueTask {
        StoredQueueTask {
            task_id: model.task_id,
            run_id: model.run_id,
            state_name: model.state_name,
            resource: model.resource,
            task_payload: model.task_payload.and_then(|s| serde_json::from_str(&s).ok()),
            status: model.status,
            attempts: model.attempts,
            max_attempts: model.max_attempts,
            priority: model.priority,
            timeout_seconds: model.timeout_seconds,
            error_message: model.error_message,
            last_error_at: model.last_error_at,
            next_retry_at: model.next_retry_at,
            queued_at: model.queued_at,
            processing_at: model.processing_at,
            completed_at: model.completed_at,
            failed_at: model.failed_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    // entity -> model
    fn to_model(entity: &StoredQueueTask) -> QueueTask {
        QueueTask {
            task_id: entity.task_id.clone(),
            run_id: entity.run_id.clone(),
            state_name: entity.state_name.clone(),
            resource: entity.resource.clone(),
            task_payload: entity.task_payload.as_ref().map(|v| v.to_string()),
            status: entity.status.clone(),
            attempts: entity.attempts,
            max_attempts: entity.max_attempts,
            priority: entity.priority,
            timeout_seconds: entity.timeout_seconds,
            error_message: entity.error_message.clone(),
            last_error_at: entity.last_error_at,
            next_retry_at: entity.next_retry_at,
            queued_at: entity.queued_at,
            processing_at: entity.processing_at,
            completed_at: entity.completed_at,
            failed_at: entity.failed_at,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }

    // entity update -> model update
    fn to_model_update(entity: &UpdateStoredQueueTask) -> UpdateQueueTask {
        UpdateQueueTask {
            status: entity.status.clone(),
            task_payload: entity.task_payload.as_ref().map(|v| v.as_ref().map(|vv| vv.to_string())),
            attempts: entity.attempts,
            priority: entity.priority,
            timeout_seconds: entity.timeout_seconds,
            error_message: entity.error_message.clone(),
            last_error_at: entity.last_error_at.clone(),
            next_retry_at: entity.next_retry_at.clone(),
            processing_at: entity.processing_at.clone(),
            completed_at: entity.completed_at.clone(),
            failed_at: entity.failed_at.clone(),
            updated_at: None,
        }
    }

    pub async fn create_queue_task(&self, task: &StoredQueueTask) -> Result<(), StorageError> {
        let model = Self::to_model(task);
        queue_task_crud::create_task(&self.pool, &model).await.map_err(StorageError::from)
    }

    pub async fn get_queue_task(&self, task_id: &str) -> Result<Option<StoredQueueTask>, StorageError> {
        let model_opt = queue_task_crud::get_task(&self.pool, task_id).await.map_err(StorageError::from)?;
        Ok(model_opt.map(Self::to_entity))
    }

    pub async fn update_queue_task(&self, task_id: &str, changes: &UpdateStoredQueueTask) -> Result<(), StorageError> {
        let model_update = Self::to_model_update(changes);
        queue_task_crud::update_task(&self.pool, task_id, &model_update).await.map_err(StorageError::from)
    }

    pub async fn delete_queue_task(&self, task_id: &str) -> Result<(), StorageError> {
        queue_task_crud::delete_task(&self.pool, task_id).await.map_err(StorageError::from)
    }

    pub async fn find_queue_tasks_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<StoredQueueTask>, StorageError> {
        let models = queue_task_crud::find_tasks_by_status(&self.pool, status, limit, offset).await.map_err(StorageError::from)?;
        Ok(models.into_iter().map(Self::to_entity).collect())
    }

    pub async fn find_queue_tasks_to_retry(&self, before: NaiveDateTime, limit: i64) -> Result<Vec<StoredQueueTask>, StorageError> {
        let models = sqlx::query_as!(
            QueueTask,
            r#"
            SELECT task_id as "task_id!",
                   run_id as "run_id!",
                   state_name as "state_name!",
                   resource as "resource!",
                   task_payload,
                   status as "status!",
                   attempts as "attempts!",
                   max_attempts as "max_attempts!",
                   priority,
                   timeout_seconds,
                   error_message,
                   last_error_at,
                   next_retry_at,
                   queued_at as "queued_at!",
                   processing_at,
                   completed_at,
                   failed_at,
                   created_at as "created_at!",
                   updated_at as "updated_at!"
            FROM queue_tasks
            WHERE status = 'retrying' AND next_retry_at <= ?
            ORDER BY next_retry_at ASC
            LIMIT ?
            "#,
            before,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(StorageError::from)?;
        Ok(models.into_iter().map(Self::to_entity).collect())
    }

    pub async fn get_task_by_run_state(&self, run_id: &str, state_name: &str) -> Result<Option<StoredQueueTask>, StorageError> {
        let model_opt = queue_task_crud::get_task_by_run_state(&self.pool, run_id, state_name).await.map_err(StorageError::from)?;
        Ok(model_opt.map(Self::to_entity))
    }

    pub async fn update_task_by_run_state(&self, run_id: &str, state_name: &str, expected_status: Option<&str>, changes: &UpdateStoredQueueTask) -> Result<u64, StorageError> {
        let model_update = Self::to_model_update(changes);
        queue_task_crud::update_task_by_run_state(&self.pool, run_id, state_name, expected_status, &model_update).await.map_err(StorageError::from)
    }
}
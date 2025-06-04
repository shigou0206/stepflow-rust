use sqlx::SqlitePool;
use crate::{
    crud::activity_task_crud,
    models::activity_task::{ActivityTask, UpdateActivityTask},
};
use stepflow_storage::entities::activity_task::{StoredActivityTask, UpdateStoredActivityTask};
use stepflow_storage::error::StorageError;

#[derive(Clone)]
pub struct ActivityTaskPersistence {
    pool: SqlitePool,
}

impl ActivityTaskPersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // models -> entity 转换
    fn to_entity(model: ActivityTask) -> StoredActivityTask {
        StoredActivityTask {
            task_token: model.task_token,
            run_id: model.run_id,
            shard_id: model.shard_id,
            seq: model.seq,
            activity_type: model.activity_type,
            state_name: model.state_name,
            input: model.input.and_then(|s| serde_json::from_str(&s).ok()),
            result: model.result.and_then(|s| serde_json::from_str(&s).ok()),
            status: model.status,
            error: model.error,
            error_details: model.error_details,
            attempt: model.attempt,
            max_attempts: model.max_attempts,
            heartbeat_at: model.heartbeat_at,
            scheduled_at: model.scheduled_at,
            started_at: model.started_at,
            completed_at: model.completed_at,
            timeout_seconds: model.timeout_seconds,
            retry_policy: model.retry_policy,
            version: model.version,
        }
    }

    // entity -> models 转换
    fn to_model(entity: &StoredActivityTask) -> ActivityTask {
        ActivityTask {
            task_token: entity.task_token.clone(),
            run_id: entity.run_id.clone(),
            shard_id: entity.shard_id,
            seq: entity.seq,
            activity_type: entity.activity_type.clone(),
            state_name: entity.state_name.clone(),
            input: entity.input.as_ref().map(|v| v.to_string()),
            result: entity.result.as_ref().map(|v| v.to_string()),
            status: entity.status.clone(),
            error: entity.error.clone(),
            error_details: entity.error_details.clone(),
            attempt: entity.attempt,
            max_attempts: entity.max_attempts,
            heartbeat_at: entity.heartbeat_at,
            scheduled_at: entity.scheduled_at,
            started_at: entity.started_at,
            completed_at: entity.completed_at,
            timeout_seconds: entity.timeout_seconds,
            retry_policy: entity.retry_policy.clone(),
            version: entity.version,
        }
    }

    // entity update -> model update
    fn to_model_update(entity: &UpdateStoredActivityTask) -> UpdateActivityTask {
        UpdateActivityTask {
            state_name: entity.state_name.clone(),
            input: entity.input.as_ref().map(|v| v.as_ref().map(|vv| vv.to_string())),
            result: entity.result.as_ref().map(|v| v.as_ref().map(|vv| vv.to_string())),
            status: entity.status.clone(),
            error: entity.error.clone(),
            error_details: entity.error_details.clone(),
            attempt: entity.attempt,
            heartbeat_at: entity.heartbeat_at.clone(),
            started_at: entity.started_at.clone(),
            completed_at: entity.completed_at.clone(),
            version: entity.version,
        }
    }

    pub async fn create_task(&self, task: &StoredActivityTask) -> Result<(), StorageError> {
        let model = Self::to_model(task);
        activity_task_crud::create_task(&self.pool, &model).await.map_err(StorageError::from)
    }

    pub async fn get_task(&self, task_token: &str) -> Result<Option<StoredActivityTask>, StorageError> {
        let model_opt = activity_task_crud::get_task(&self.pool, task_token).await.map_err(StorageError::from)?;
        Ok(model_opt.map(Self::to_entity))
    }

    pub async fn find_tasks_by_status(
        &self,
        status: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredActivityTask>, StorageError> {
        let models = activity_task_crud::find_tasks_by_status(&self.pool, status, limit, offset).await.map_err(StorageError::from)?;
        Ok(models.into_iter().map(Self::to_entity).collect())
    }

    pub async fn update_task(
        &self,
        task_token: &str,
        changes: &UpdateStoredActivityTask,
    ) -> Result<(), StorageError> {
        let model_update = Self::to_model_update(changes);
        activity_task_crud::update_task(&self.pool, task_token, &model_update).await.map_err(StorageError::from)
    }

    pub async fn delete_task(&self, task_token: &str) -> Result<(), StorageError> {
        activity_task_crud::delete_task(&self.pool, task_token).await.map_err(StorageError::from)
    }
}
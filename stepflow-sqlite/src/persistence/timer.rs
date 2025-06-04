use sqlx::SqlitePool;
use crate::{
    crud::timer_crud,
    models::timer::{Timer, UpdateTimer},
};
use stepflow_storage::entities::timer::{StoredTimer, UpdateStoredTimer};
use stepflow_storage::error::StorageError;
use chrono::NaiveDateTime;
use serde_json;

#[derive(Clone)]
pub struct TimerPersistence {
    pool: SqlitePool,
}

impl TimerPersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // model -> entity
    fn to_entity(model: Timer) -> StoredTimer {
        StoredTimer {
            timer_id: model.timer_id,
            run_id: model.run_id,
            shard_id: model.shard_id,
            fire_at: model.fire_at,
            status: model.status,
            version: model.version,
            state_name: model.state_name,
            payload: model.payload.and_then(|s| serde_json::from_str(&s).ok()),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    // entity -> model
    fn to_model(entity: &StoredTimer) -> Timer {
        Timer {
            timer_id: entity.timer_id.clone(),
            run_id: entity.run_id.clone(),
            shard_id: entity.shard_id,
            fire_at: entity.fire_at,
            status: entity.status.clone(),
            version: entity.version,
            state_name: entity.state_name.clone(),
            payload: entity.payload.as_ref().map(|v| v.to_string()),
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }

    // entity update -> model update
    fn to_model_update(entity: &UpdateStoredTimer) -> UpdateTimer {
        UpdateTimer {
            fire_at: entity.fire_at,
            status: entity.status.clone(),
            version: entity.version,
            state_name: None,
            payload: entity.payload.as_ref().map(|v| v.as_ref().map(|vv| vv.to_string())),
        }
    }

    pub async fn create_timer(&self, timer: &StoredTimer) -> Result<(), StorageError> {
        let model = Self::to_model(timer);
        timer_crud::create_timer(&self.pool, &model).await.map_err(StorageError::from)
    }

    pub async fn get_timer(&self, timer_id: &str) -> Result<Option<StoredTimer>, StorageError> {
        let model_opt = timer_crud::get_timer(&self.pool, timer_id).await.map_err(StorageError::from)?;
        Ok(model_opt.map(Self::to_entity))
    }

    pub async fn update_timer(&self, timer_id: &str, changes: &UpdateStoredTimer) -> Result<(), StorageError> {
        let model_update = Self::to_model_update(changes);
        timer_crud::update_timer(&self.pool, timer_id, &model_update).await.map_err(StorageError::from)
    }

    pub async fn delete_timer(&self, timer_id: &str) -> Result<(), StorageError> {
        timer_crud::delete_timer(&self.pool, timer_id).await.map_err(StorageError::from)
    }

    pub async fn find_timers_before(&self, before: NaiveDateTime, limit: i64) -> Result<Vec<StoredTimer>, StorageError> {
        let models = timer_crud::find_timers_before(&self.pool, before, limit).await.map_err(StorageError::from)?;
        Ok(models.into_iter().map(Self::to_entity).collect())
    }
}
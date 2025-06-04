use sqlx::SqlitePool;
use crate::{
    crud::workflow_event_crud,
    models::workflow_event::{WorkflowEvent, UpdateWorkflowEvent},
};
use stepflow_storage::entities::workflow_event::{StoredWorkflowEvent, UpdateStoredWorkflowEvent};
use stepflow_storage::error::StorageError;

#[derive(Clone)]
pub struct WorkflowEventPersistence {
    pool: SqlitePool,
}

impl WorkflowEventPersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // model -> entity
    fn to_entity(model: WorkflowEvent) -> StoredWorkflowEvent {
        StoredWorkflowEvent {
            id: model.id,
            run_id: model.run_id,
            shard_id: model.shard_id,
            event_id: model.event_id,
            event_type: model.event_type,
            state_id: model.state_id,
            state_type: model.state_type,
            trace_id: model.trace_id,
            parent_event_id: model.parent_event_id,
            context_version: model.context_version,
            attributes: model.attributes,
            attr_version: model.attr_version,
            timestamp: model.timestamp,
            archived: model.archived,
        }
    }

    // entity -> model
    fn to_model(entity: &StoredWorkflowEvent) -> WorkflowEvent {
        WorkflowEvent {
            id: entity.id,
            run_id: entity.run_id.clone(),
            shard_id: entity.shard_id,
            event_id: entity.event_id,
            event_type: entity.event_type.clone(),
            state_id: entity.state_id.clone(),
            state_type: entity.state_type.clone(),
            trace_id: entity.trace_id.clone(),
            parent_event_id: entity.parent_event_id,
            context_version: entity.context_version,
            attributes: entity.attributes.clone(),
            attr_version: entity.attr_version,
            timestamp: entity.timestamp,
            archived: entity.archived,
        }
    }

    // entity update -> model update
    fn to_model_update(entity: &UpdateStoredWorkflowEvent) -> UpdateWorkflowEvent {
        UpdateWorkflowEvent {
            event_type: entity.event_type.clone(),
            state_id: entity.state_id.clone(),
            state_type: entity.state_type.clone(),
            trace_id: entity.trace_id.clone(),
            parent_event_id: entity.parent_event_id,
            context_version: entity.context_version,
            attributes: entity.attributes.clone(),
            attr_version: entity.attr_version,
            timestamp: entity.timestamp,
            archived: entity.archived,
        }
    }

    pub async fn create_event(&self, event: &StoredWorkflowEvent) -> Result<i64, StorageError> {
        let model = Self::to_model(event);
        workflow_event_crud::create_event(&self.pool, &model)
            .await
            .map_err(StorageError::from)
    }

    pub async fn get_event(&self, id: i64) -> Result<Option<StoredWorkflowEvent>, StorageError> {
        let model_opt = workflow_event_crud::get_event(&self.pool, id)
            .await
            .map_err(StorageError::from)?;
        Ok(model_opt.map(Self::to_entity))
    }

    pub async fn find_events_by_run_id(
        &self,
        run_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredWorkflowEvent>, StorageError> {
        let models = workflow_event_crud::find_events_by_run_id(&self.pool, run_id, limit, offset)
            .await
            .map_err(StorageError::from)?;
        Ok(models.into_iter().map(Self::to_entity).collect())
    }

    pub async fn update_event(&self, id: i64, changes: &UpdateStoredWorkflowEvent) -> Result<(), StorageError> {
        let model_update = Self::to_model_update(changes);
        workflow_event_crud::update_event(&self.pool, id, &model_update)
            .await
            .map_err(StorageError::from)
    }

    pub async fn archive_event(&self, id: i64) -> Result<(), StorageError> {
        workflow_event_crud::archive_event(&self.pool, id)
            .await
            .map_err(StorageError::from)
    }

    pub async fn delete_event(&self, id: i64) -> Result<(), StorageError> {
        workflow_event_crud::delete_event(&self.pool, id)
            .await
            .map_err(StorageError::from)
    }

    pub async fn delete_events_by_run_id(&self, run_id: &str) -> Result<u64, StorageError> {
        workflow_event_crud::delete_events_by_run_id(&self.pool, run_id)
            .await
            .map_err(StorageError::from)
    }
}
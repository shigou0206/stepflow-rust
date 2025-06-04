use sqlx::SqlitePool;
use crate::{
    crud::workflow_state_crud,
    models::workflow_state::{WorkflowState, UpdateWorkflowState},
};
use stepflow_storage::entities::workflow_state::{StoredWorkflowState, UpdateStoredWorkflowState};
use stepflow_storage::error::StorageError;

#[derive(Clone)]
pub struct WorkflowStatePersistence {
    pool: SqlitePool,
}

impl WorkflowStatePersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // model -> entity
    fn to_entity(model: WorkflowState) -> StoredWorkflowState {
        StoredWorkflowState {
            state_id: model.state_id,
            run_id: model.run_id,
            shard_id: model.shard_id,
            state_name: model.state_name,
            state_type: model.state_type,
            status: model.status,
            input: model.input.and_then(|s| serde_json::from_str(&s).ok()),
            output: model.output.and_then(|s| serde_json::from_str(&s).ok()),
            error: model.error,
            error_details: model.error_details,
            started_at: model.started_at,
            completed_at: model.completed_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
            version: model.version,
        }
    }

    // entity -> model
    fn to_model(entity: &StoredWorkflowState) -> WorkflowState {
        WorkflowState {
            state_id: entity.state_id.clone(),
            run_id: entity.run_id.clone(),
            shard_id: entity.shard_id,
            state_name: entity.state_name.clone(),
            state_type: entity.state_type.clone(),
            status: entity.status.clone(),
            input: entity.input.as_ref().map(|v| v.to_string()),
            output: entity.output.as_ref().map(|v| v.to_string()),
            error: entity.error.clone(),
            error_details: entity.error_details.clone(),
            started_at: entity.started_at,
            completed_at: entity.completed_at,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            version: entity.version,
        }
    }

    // entity update -> model update
    fn to_model_update(entity: &UpdateStoredWorkflowState) -> UpdateWorkflowState {
        UpdateWorkflowState {
            state_name: entity.state_name.clone(),
            state_type: entity.state_type.clone(),
            status: entity.status.clone(),
            input: entity.input.as_ref().map(|v| v.as_ref().map(|vv| vv.to_string())),
            output: entity.output.as_ref().map(|v| v.as_ref().map(|vv| vv.to_string())),
            error: entity.error.clone(),
            error_details: entity.error_details.clone(),
            started_at: entity.started_at.clone(),
            completed_at: entity.completed_at.clone(),
            version: entity.version,
        }
    }

    pub async fn create_state(&self, state: &StoredWorkflowState) -> Result<(), StorageError> {
        let model = Self::to_model(state);
        workflow_state_crud::create_state(&self.pool, &model).await.map_err(StorageError::from)
    }

    pub async fn get_state(&self, state_id: &str) -> Result<Option<StoredWorkflowState>, StorageError> {
        let model_opt = workflow_state_crud::get_state(&self.pool, state_id).await.map_err(StorageError::from)?;
        Ok(model_opt.map(Self::to_entity))
    }

    pub async fn find_states_by_run_id(
        &self,
        run_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredWorkflowState>, StorageError> {
        let models = workflow_state_crud::find_states_by_run_id(&self.pool, run_id, limit, offset).await.map_err(StorageError::from)?;
        Ok(models.into_iter().map(Self::to_entity).collect())
    }

    pub async fn update_state(
        &self,
        state_id: &str,
        changes: &UpdateStoredWorkflowState,
    ) -> Result<(), StorageError> {
        let model_update = Self::to_model_update(changes);
        workflow_state_crud::update_state(&self.pool, state_id, &model_update).await.map_err(StorageError::from)
    }

    pub async fn delete_state(&self, state_id: &str) -> Result<(), StorageError> {
        workflow_state_crud::delete_state(&self.pool, state_id).await.map_err(StorageError::from)
    }
}
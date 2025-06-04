use sqlx::SqlitePool;
use crate::{
    crud::workflow_visibility_crud,
    models::workflow_visibility::{WorkflowVisibility, UpdateWorkflowVisibility},
};
use stepflow_storage::entities::workflow_visibility::{StoredWorkflowVisibility, UpdateStoredWorkflowVisibility};
use stepflow_storage::error::StorageError;

#[derive(Clone)]
pub struct WorkflowVisibilityPersistence {
    pool: SqlitePool,
}

impl WorkflowVisibilityPersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // model -> entity
    fn to_entity(model: WorkflowVisibility) -> StoredWorkflowVisibility {
        StoredWorkflowVisibility {
            run_id: model.run_id,
            workflow_id: model.workflow_id,
            workflow_type: model.workflow_type,
            start_time: model.start_time,
            close_time: model.close_time,
            status: model.status,
            memo: model.memo,
            search_attrs: model.search_attrs.and_then(|s| serde_json::from_str(&s).ok()),
            version: model.version,
        }
    }

    // entity -> model
    fn to_model(entity: &StoredWorkflowVisibility) -> WorkflowVisibility {
        WorkflowVisibility {
            run_id: entity.run_id.clone(),
            workflow_id: entity.workflow_id.clone(),
            workflow_type: entity.workflow_type.clone(),
            start_time: entity.start_time,
            close_time: entity.close_time,
            status: entity.status.clone(),
            memo: entity.memo.clone(),
            search_attrs: entity.search_attrs.as_ref().map(|v| v.to_string()),
            version: entity.version,
        }
    }

    // entity update -> model update
    fn to_model_update(entity: &UpdateStoredWorkflowVisibility) -> UpdateWorkflowVisibility {
        UpdateWorkflowVisibility {
            workflow_id: entity.workflow_id.clone(),
            workflow_type: entity.workflow_type.clone(),
            start_time: entity.start_time.clone(),
            close_time: entity.close_time.clone(),
            status: entity.status.clone(),
            memo: entity.memo.clone(),
            search_attrs: entity.search_attrs.as_ref().map(|v| v.as_ref().map(|vv| vv.to_string())),
            version: entity.version,
        }
    }

    pub async fn create_visibility(&self, vis: &StoredWorkflowVisibility) -> Result<(), StorageError> {
        let model = Self::to_model(vis);
        workflow_visibility_crud::create_visibility(&self.pool, &model).await.map_err(StorageError::from)
    }

    pub async fn get_visibility(&self, run_id: &str) -> Result<Option<StoredWorkflowVisibility>, StorageError> {
        let model_opt = workflow_visibility_crud::get_visibility(&self.pool, run_id).await.map_err(StorageError::from)?;
        Ok(model_opt.map(Self::to_entity))
    }

    pub async fn find_visibilities_by_status(
        &self,
        status: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredWorkflowVisibility>, StorageError> {
        let models = workflow_visibility_crud::find_visibilities_by_status(&self.pool, status, limit, offset)
            .await
            .map_err(StorageError::from)?;
        Ok(models.into_iter().map(Self::to_entity).collect())
    }

    pub async fn update_visibility(
        &self,
        run_id: &str,
        changes: &UpdateStoredWorkflowVisibility,
    ) -> Result<(), StorageError> {
        let model_update = Self::to_model_update(changes);
        workflow_visibility_crud::update_visibility(&self.pool, run_id, &model_update)
            .await
            .map_err(StorageError::from)
    }

    pub async fn delete_visibility(&self, run_id: &str) -> Result<(), StorageError> {
        workflow_visibility_crud::delete_visibility(&self.pool, run_id).await.map_err(StorageError::from)
    }
}
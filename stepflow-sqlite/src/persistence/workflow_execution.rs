use crate::{
    crud::workflow_execution_crud,
    models::workflow_execution::{UpdateWorkflowExecution, WorkflowExecution},
};
use sqlx::SqlitePool;
use stepflow_storage::entities::workflow_execution::{
    StoredWorkflowExecution, UpdateStoredWorkflowExecution,
};
use stepflow_storage::error::StorageError;

#[derive(Clone)]
pub struct WorkflowExecutionPersistence {
    pool: SqlitePool,
}

impl WorkflowExecutionPersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // model -> entity
    fn to_entity(model: WorkflowExecution) -> StoredWorkflowExecution {
        StoredWorkflowExecution {
            run_id: model.run_id,
            workflow_id: model.workflow_id,
            shard_id: model.shard_id,
            template_id: model.template_id,
            mode: model.mode,
            current_state_name: model.current_state_name,
            status: model.status,
            workflow_type: model.workflow_type,
            input: model.input.and_then(|s| serde_json::from_str(&s).ok()),
            input_version: model.input_version,
            result: model.result.and_then(|s| serde_json::from_str(&s).ok()),
            result_version: model.result_version,
            start_time: model.start_time,
            close_time: model.close_time,
            current_event_id: model.current_event_id,
            memo: model.memo,
            search_attrs: model
                .search_attrs
                .and_then(|s| serde_json::from_str(&s).ok()),
            context_snapshot: model
                .context_snapshot
                .and_then(|s| serde_json::from_str(&s).ok()),
            version: model.version,

            parent_run_id: model.parent_run_id,
            parent_state_name: model.parent_state_name,
            dsl_definition: model
                .dsl_definition
                .and_then(|s| serde_json::from_str(&s).ok()),
        }
    }

    // entity -> model
    fn to_model(entity: &StoredWorkflowExecution) -> WorkflowExecution {
        WorkflowExecution {
            run_id: entity.run_id.clone(),
            workflow_id: entity.workflow_id.clone(),
            shard_id: entity.shard_id,
            template_id: entity.template_id.clone(),
            mode: entity.mode.clone(),
            current_state_name: entity.current_state_name.clone(),
            status: entity.status.clone(),
            workflow_type: entity.workflow_type.clone(),
            input: entity.input.as_ref().map(|v| v.to_string()),
            input_version: entity.input_version,
            result: entity.result.as_ref().map(|v| v.to_string()),
            result_version: entity.result_version,
            start_time: entity.start_time,
            close_time: entity.close_time,
            current_event_id: entity.current_event_id,
            memo: entity.memo.clone(),
            search_attrs: entity.search_attrs.as_ref().map(|v| v.to_string()),
            context_snapshot: entity.context_snapshot.as_ref().map(|v| v.to_string()),
            version: entity.version,

            parent_run_id: entity.parent_run_id.clone(),
            parent_state_name: entity.parent_state_name.clone(),
            dsl_definition: entity.dsl_definition.as_ref().map(|v| v.to_string()),
        }
    }

    // entity update -> model update
    fn to_model_update(entity: &UpdateStoredWorkflowExecution) -> UpdateWorkflowExecution {
        UpdateWorkflowExecution {
            workflow_id: entity.workflow_id.clone(),
            shard_id: entity.shard_id,
            template_id: entity.template_id.clone(),
            mode: entity.mode.clone(),
            current_state_name: entity.current_state_name.clone(),
            status: entity.status.clone(),
            workflow_type: entity.workflow_type.clone(),
            input: entity
                .input
                .as_ref()
                .map(|v| v.as_ref().map(|vv| vv.to_string())),
            input_version: entity.input_version,
            result: entity
                .result
                .as_ref()
                .map(|v| v.as_ref().map(|vv| vv.to_string())),
            result_version: entity.result_version,
            start_time: entity.start_time,
            close_time: entity.close_time.clone(),
            current_event_id: entity.current_event_id,
            memo: entity.memo.clone(),
            search_attrs: entity
                .search_attrs
                .as_ref()
                .map(|v| v.as_ref().map(|vv| vv.to_string())),
            context_snapshot: entity
                .context_snapshot
                .as_ref()
                .map(|v| v.as_ref().map(|vv| vv.to_string())),
            version: entity.version,

            parent_run_id: entity.parent_run_id.clone(),
            parent_state_name: entity.parent_state_name.clone(),
            dsl_definition: entity
                .dsl_definition
                .as_ref()
                .map(|v| v.as_ref().map(|vv| vv.to_string())),
        }
    }

    pub async fn create_execution(
        &self,
        exec: &StoredWorkflowExecution,
    ) -> Result<(), StorageError> {
        let model = Self::to_model(exec);
        workflow_execution_crud::create_execution(&self.pool, &model)
            .await
            .map_err(StorageError::from)
    }

    pub async fn get_execution(
        &self,
        run_id: &str,
    ) -> Result<Option<StoredWorkflowExecution>, StorageError> {
        let model_opt = workflow_execution_crud::get_execution(&self.pool, run_id)
            .await
            .map_err(StorageError::from)?;
        Ok(model_opt.map(Self::to_entity))
    }

    pub async fn find_executions(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredWorkflowExecution>, StorageError> {
        let models = workflow_execution_crud::find_executions(&self.pool, limit, offset)
            .await
            .map_err(StorageError::from)?;
        Ok(models.into_iter().map(Self::to_entity).collect())
    }

    pub async fn find_executions_by_status(
        &self,
        status: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredWorkflowExecution>, StorageError> {
        let models =
            workflow_execution_crud::find_executions_by_status(&self.pool, status, limit, offset)
                .await
                .map_err(StorageError::from)?;
        Ok(models.into_iter().map(Self::to_entity).collect())
    }

    pub async fn update_execution(
        &self,
        run_id: &str,
        changes: &UpdateStoredWorkflowExecution,
    ) -> Result<(), StorageError> {
        let model_update = Self::to_model_update(changes);
        workflow_execution_crud::update_execution(&self.pool, run_id, &model_update)
            .await
            .map_err(StorageError::from)
    }

    pub async fn delete_execution(&self, run_id: &str) -> Result<(), StorageError> {
        workflow_execution_crud::delete_execution(&self.pool, run_id)
            .await
            .map_err(StorageError::from)
    }
}

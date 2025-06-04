use sqlx::SqlitePool;
use crate::{
    crud::workflow_template_crud,
    models::workflow_template::{WorkflowTemplate, UpdateWorkflowTemplate},
};
use stepflow_storage::entities::workflow_template::{StoredWorkflowTemplate, UpdateStoredWorkflowTemplate};
use stepflow_storage::error::StorageError;

#[derive(Clone)]
pub struct WorkflowTemplatePersistence {
    pool: SqlitePool,
}

impl WorkflowTemplatePersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // model -> entity
    fn to_entity(model: WorkflowTemplate) -> StoredWorkflowTemplate {
        StoredWorkflowTemplate {
            template_id: model.template_id,
            name: model.name,
            description: model.description,
            dsl_definition: model.dsl_definition,
            version: model.version,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    // entity -> model
    fn to_model(entity: &StoredWorkflowTemplate) -> WorkflowTemplate {
        WorkflowTemplate {
            template_id: entity.template_id.clone(),
            name: entity.name.clone(),
            description: entity.description.clone(),
            dsl_definition: entity.dsl_definition.clone(),
            version: entity.version,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }

    // entity update -> model update
    fn to_model_update(entity: &UpdateStoredWorkflowTemplate) -> UpdateWorkflowTemplate {
        UpdateWorkflowTemplate {
            name: entity.name.clone(),
            description: entity.description.clone(),
            dsl_definition: entity.dsl_definition.clone(),
            version: entity.version,
        }
    }

    pub async fn create_template(&self, tpl: &StoredWorkflowTemplate) -> Result<(), StorageError> {
        let model = Self::to_model(tpl);
        workflow_template_crud::create_template(&self.pool, &model).await.map_err(StorageError::from)
    }

    pub async fn get_template(&self, template_id: &str) -> Result<Option<StoredWorkflowTemplate>, StorageError> {
        let model_opt = workflow_template_crud::get_template(&self.pool, template_id).await.map_err(StorageError::from)?;
        Ok(model_opt.map(Self::to_entity))
    }

    pub async fn find_templates(&self, limit: i64, offset: i64) -> Result<Vec<StoredWorkflowTemplate>, StorageError> {
        let models = workflow_template_crud::find_templates(&self.pool, limit, offset).await.map_err(StorageError::from)?;
        Ok(models.into_iter().map(Self::to_entity).collect())
    }

    pub async fn update_template(
        &self,
        template_id: &str,
        changes: &UpdateStoredWorkflowTemplate,
    ) -> Result<(), StorageError> {
        let model_update = Self::to_model_update(changes);
        workflow_template_crud::update_template(&self.pool, template_id, &model_update).await.map_err(StorageError::from)
    }

    pub async fn delete_template(&self, template_id: &str) -> Result<(), StorageError> {
        workflow_template_crud::delete_template(&self.pool, template_id).await.map_err(StorageError::from)
    }
}
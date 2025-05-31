use sqlx::SqlitePool;
use stepflow_sqlite::{
    crud::workflow_template_crud,
    models::workflow_template::{WorkflowTemplate, UpdateWorkflowTemplate},
};

#[derive(Clone)]
pub struct WorkflowTemplatePersistence {
    pool: SqlitePool,
}

impl WorkflowTemplatePersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_template(&self, tpl: &WorkflowTemplate) -> Result<(), sqlx::Error> {
        workflow_template_crud::create_template(&self.pool, tpl).await
    }

    pub async fn get_template(&self, template_id: &str) -> Result<Option<WorkflowTemplate>, sqlx::Error> {
        workflow_template_crud::get_template(&self.pool, template_id).await
    }

    pub async fn find_templates(&self, limit: i64, offset: i64) -> Result<Vec<WorkflowTemplate>, sqlx::Error> {
        workflow_template_crud::find_templates(&self.pool, limit, offset).await
    }

    pub async fn update_template(&self, template_id: &str, changes: &UpdateWorkflowTemplate) -> Result<(), sqlx::Error> {
        workflow_template_crud::update_template(&self.pool, template_id, changes).await
    }

    pub async fn delete_template(&self, template_id: &str) -> Result<(), sqlx::Error> {
        workflow_template_crud::delete_template(&self.pool, template_id).await
    }
}
use sqlx::SqlitePool;
use stepflow_sqlite::{
    crud::workflow_visibility_crud,
    models::workflow_visibility::{WorkflowVisibility, UpdateWorkflowVisibility},
};

#[derive(Clone)]
pub struct WorkflowVisibilityPersistence {
    pool: SqlitePool,
}

impl WorkflowVisibilityPersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_visibility(&self, vis: &WorkflowVisibility) -> Result<(), sqlx::Error> {
        workflow_visibility_crud::create_visibility(&self.pool, vis).await
    }

    pub async fn get_visibility(&self, run_id: &str) -> Result<Option<WorkflowVisibility>, sqlx::Error> {
        workflow_visibility_crud::get_visibility(&self.pool, run_id).await
    }

    pub async fn find_visibilities_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<WorkflowVisibility>, sqlx::Error> {
        workflow_visibility_crud::find_visibilities_by_status(&self.pool, status, limit, offset).await
    }

    pub async fn update_visibility(&self, run_id: &str, changes: &UpdateWorkflowVisibility) -> Result<(), sqlx::Error> {
        workflow_visibility_crud::update_visibility(&self.pool, run_id, changes).await
    }

    pub async fn delete_visibility(&self, run_id: &str) -> Result<(), sqlx::Error> {
        workflow_visibility_crud::delete_visibility(&self.pool, run_id).await
    }
}
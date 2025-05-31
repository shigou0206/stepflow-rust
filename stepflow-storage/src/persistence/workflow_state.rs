use sqlx::SqlitePool;
use stepflow_sqlite::{
    crud::workflow_state_crud,
    models::workflow_state::{WorkflowState, UpdateWorkflowState},
};

#[derive(Clone)]
pub struct WorkflowStatePersistence {
    pool: SqlitePool,
}

impl WorkflowStatePersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_state(&self, state: &WorkflowState) -> Result<(), sqlx::Error> {
        workflow_state_crud::create_state(&self.pool, state).await
    }

    pub async fn get_state(&self, state_id: &str) -> Result<Option<WorkflowState>, sqlx::Error> {
        workflow_state_crud::get_state(&self.pool, state_id).await
    }

    pub async fn find_states_by_run_id(
        &self,
        run_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<WorkflowState>, sqlx::Error> {
        workflow_state_crud::find_states_by_run_id(&self.pool, run_id, limit, offset).await
    }

    pub async fn update_state(
        &self,
        state_id: &str,
        changes: &UpdateWorkflowState,
    ) -> Result<(), sqlx::Error> {
        workflow_state_crud::update_state(&self.pool, state_id, changes).await
    }

    pub async fn delete_state(&self, state_id: &str) -> Result<(), sqlx::Error> {
        workflow_state_crud::delete_state(&self.pool, state_id).await
    }
}
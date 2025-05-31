use sqlx::SqlitePool;
use stepflow_sqlite::{
    crud::workflow_execution_crud,
    models::workflow_execution::{WorkflowExecution, UpdateWorkflowExecution},
};

#[derive(Clone)]
pub struct WorkflowExecutionPersistence {
    pool: SqlitePool,
}

impl WorkflowExecutionPersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, exec: &WorkflowExecution) -> Result<(), sqlx::Error> {
        workflow_execution_crud::create_execution(&self.pool, exec).await
    }

    pub async fn get(&self, run_id: &str) -> Result<Option<WorkflowExecution>, sqlx::Error> {
        workflow_execution_crud::get_execution(&self.pool, run_id).await
    }

    pub async fn find(&self, limit: i64, offset: i64) -> Result<Vec<WorkflowExecution>, sqlx::Error> {
        workflow_execution_crud::find_executions(&self.pool, limit, offset).await
    }

    pub async fn find_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<WorkflowExecution>, sqlx::Error> {
        workflow_execution_crud::find_executions_by_status(&self.pool, status, limit, offset).await
    }

    pub async fn update(
        &self,
        run_id: &str,
        changes: &UpdateWorkflowExecution,
    ) -> Result<(), sqlx::Error> {
        workflow_execution_crud::update_execution(&self.pool, run_id, changes).await
    }

    pub async fn delete(&self, run_id: &str) -> Result<(), sqlx::Error> {
        workflow_execution_crud::delete_execution(&self.pool, run_id).await
    }
}
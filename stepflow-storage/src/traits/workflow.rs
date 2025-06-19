use crate::entities::workflow_execution::{StoredWorkflowExecution, UpdateStoredWorkflowExecution};
use crate::error::StorageError;

#[async_trait::async_trait]
pub trait WorkflowStorage: Send + Sync {
    /// Create a new workflow execution
    async fn create_execution(&self, exec: &StoredWorkflowExecution) -> Result<(), StorageError>;

    /// Get a workflow execution by run_id
    async fn get_execution(
        &self,
        run_id: &str,
    ) -> Result<Option<StoredWorkflowExecution>, StorageError>;

    /// Find workflow executions with pagination
    async fn find_executions(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredWorkflowExecution>, StorageError>;

    /// Find workflow executions by status with pagination
    async fn find_executions_by_status(
        &self,
        status: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredWorkflowExecution>, StorageError>;

    /// Update a workflow execution
    async fn update_execution(
        &self,
        run_id: &str,
        changes: &UpdateStoredWorkflowExecution,
    ) -> Result<(), StorageError>;

    /// Delete a workflow execution
    async fn delete_execution(&self, run_id: &str) -> Result<(), StorageError>;

    /// Find all subflows for a given parent_run_id and state_name
    async fn find_subflows_by_parent(
        &self,
        parent_run_id: &str,
        parent_state_name: &str,
    ) -> Result<Vec<StoredWorkflowExecution>, StorageError>;
}

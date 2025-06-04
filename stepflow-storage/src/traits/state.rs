use crate::error::StorageError;
use crate::entities::workflow_state::{StoredWorkflowState, UpdateStoredWorkflowState};

#[async_trait::async_trait]
pub trait StateStorage: Send + Sync {
    /// Create a new workflow state
    async fn create_state(&self, state: &StoredWorkflowState) -> Result<(), StorageError>;
    
    /// Get a workflow state by state_id
    async fn get_state(&self, state_id: &str) -> Result<Option<StoredWorkflowState>, StorageError>;
    
    /// Find workflow states by run_id with pagination
    async fn find_states_by_run_id(&self, run_id: &str, limit: i64, offset: i64) -> Result<Vec<StoredWorkflowState>, StorageError>;
    
    /// Update a workflow state
    async fn update_state(&self, state_id: &str, changes: &UpdateStoredWorkflowState) -> Result<(), StorageError>;
    
    /// Delete a workflow state
    async fn delete_state(&self, state_id: &str) -> Result<(), StorageError>;
} 
use crate::error::StorageError;
use crate::entities::workflow_visibility::{StoredWorkflowVisibility, UpdateStoredWorkflowVisibility};

#[async_trait::async_trait]
pub trait VisibilityStorage: Send + Sync {
    /// Create a new workflow visibility record
    async fn create_visibility(&self, vis: &StoredWorkflowVisibility) -> Result<(), StorageError>;
    
    /// Get a workflow visibility record by run_id
    async fn get_visibility(&self, run_id: &str) -> Result<Option<StoredWorkflowVisibility>, StorageError>;
    
    /// Find workflow visibility records by status with pagination
    async fn find_visibilities_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<StoredWorkflowVisibility>, StorageError>;
    
    /// Update a workflow visibility record
    async fn update_visibility(&self, run_id: &str, changes: &UpdateStoredWorkflowVisibility) -> Result<(), StorageError>;
    
    /// Delete a workflow visibility record
    async fn delete_visibility(&self, run_id: &str) -> Result<(), StorageError>;
} 
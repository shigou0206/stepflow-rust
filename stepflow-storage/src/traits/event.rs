use crate::error::StorageError;
use crate::entities::workflow_event::{StoredWorkflowEvent, UpdateStoredWorkflowEvent};

#[async_trait::async_trait]
pub trait EventStorage: Send + Sync {
    /// Create a new workflow event, returns the event ID
    async fn create_event(&self, event: &StoredWorkflowEvent) -> Result<i64, StorageError>;
    
    /// Get a workflow event by ID
    async fn get_event(&self, id: i64) -> Result<Option<StoredWorkflowEvent>, StorageError>;
    
    /// Find workflow events by run_id with pagination
    async fn find_events_by_run_id(&self, run_id: &str, limit: i64, offset: i64) -> Result<Vec<StoredWorkflowEvent>, StorageError>;
    
    /// Update a workflow event
    async fn update_event(&self, id: i64, changes: &UpdateStoredWorkflowEvent) -> Result<(), StorageError>;
    
    /// Archive a workflow event
    async fn archive_event(&self, id: i64) -> Result<(), StorageError>;
    
    /// Delete a workflow event
    async fn delete_event(&self, id: i64) -> Result<(), StorageError>;
    
    /// Delete all events for a workflow execution
    async fn delete_events_by_run_id(&self, run_id: &str) -> Result<u64, StorageError>;
} 
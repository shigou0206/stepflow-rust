use crate::error::StorageError;
use crate::entities::activity_task::{StoredActivityTask, UpdateStoredActivityTask};

#[async_trait::async_trait]
pub trait ActivityStorage: Send + Sync {
    /// Create a new activity task
    async fn create_task(&self, task: &StoredActivityTask) -> Result<(), StorageError>;
    
    /// Get an activity task by task_token
    async fn get_task(&self, task_token: &str) -> Result<Option<StoredActivityTask>, StorageError>;
    
    /// Find activity tasks by status with pagination
    async fn find_tasks_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<StoredActivityTask>, StorageError>;
    
    /// Update an activity task
    async fn update_task(&self, task_token: &str, changes: &UpdateStoredActivityTask) -> Result<(), StorageError>;
    
    /// Delete an activity task
    async fn delete_task(&self, task_token: &str) -> Result<(), StorageError>;
} 
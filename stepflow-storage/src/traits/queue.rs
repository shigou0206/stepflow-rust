use crate::error::StorageError;
use crate::entities::queue_task::{StoredQueueTask, UpdateStoredQueueTask};

#[async_trait::async_trait]
pub trait QueueStorage: Send + Sync {
    /// Create a new queue task
    async fn create_queue_task(&self, task: &StoredQueueTask) -> Result<(), StorageError>;
    
    /// Get a queue task by task_id
    async fn get_queue_task(&self, task_id: &str) -> Result<Option<StoredQueueTask>, StorageError>;
    
    /// Update a queue task
    async fn update_queue_task(&self, task_id: &str, changes: &UpdateStoredQueueTask) -> Result<(), StorageError>;
    
    /// Delete a queue task
    async fn delete_queue_task(&self, task_id: &str) -> Result<(), StorageError>;
    
    /// Find queue tasks by status with pagination
    async fn find_queue_tasks_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<StoredQueueTask>, StorageError>;

    async fn find_queue_tasks_to_retry(
        &self,
        before: chrono::NaiveDateTime,
        limit: i64,
    ) -> Result<Vec<StoredQueueTask>, StorageError>;

    async fn get_task_by_run_state(
        &self,
        run_id: &str,
        state_name: &str,
    ) -> Result<Option<StoredQueueTask>, StorageError>;

    async fn update_task_by_run_state(
        &self,
        run_id: &str,
        state_name: &str,
        expected_status: Option<&str>, 
        changes: &UpdateStoredQueueTask,
    ) -> Result<u64, StorageError>;
} 
use chrono::NaiveDateTime;
use crate::error::StorageError;
use crate::entities::timer::{StoredTimer, UpdateStoredTimer};

#[async_trait::async_trait]
pub trait TimerStorage: Send + Sync {
    /// Create a new timer
    async fn create_timer(&self, timer: &StoredTimer) -> Result<(), StorageError>;
    
    /// Get a timer by timer_id
    async fn get_timer(&self, timer_id: &str) -> Result<Option<StoredTimer>, StorageError>;
    
    /// Update a timer
    async fn update_timer(&self, timer_id: &str, changes: &UpdateStoredTimer) -> Result<(), StorageError>;
    
    /// Delete a timer
    async fn delete_timer(&self, timer_id: &str) -> Result<(), StorageError>;
    
    /// Find timers that should fire before the given time
    async fn find_timers_before(&self, before: NaiveDateTime, limit: i64) -> Result<Vec<StoredTimer>, StorageError>;
} 
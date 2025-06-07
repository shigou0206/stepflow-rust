use async_trait::async_trait;
use crate::error::StorageError;
use crate::traits::*;
use crate::transaction::TransactionManager;
use crate::entities::*;
use chrono::NaiveDateTime;

pub struct DummyPersistence;

#[async_trait]
impl WorkflowStorage for DummyPersistence {
    async fn create_execution(&self, _exec: &StoredWorkflowExecution) -> Result<(), StorageError> { unimplemented!() }
    async fn get_execution(&self, _id: &str) -> Result<Option<StoredWorkflowExecution>, StorageError> { unimplemented!() }
    async fn find_executions(&self, _start: i64, _end: i64) -> Result<Vec<StoredWorkflowExecution>, StorageError> { unimplemented!() }
    async fn find_executions_by_status(&self, _status: &str, _start: i64, _end: i64) -> Result<Vec<StoredWorkflowExecution>, StorageError> { unimplemented!() }
    async fn update_execution(&self, _id: &str, _update: &UpdateStoredWorkflowExecution) -> Result<(), StorageError> { unimplemented!() }
    async fn delete_execution(&self, _id: &str) -> Result<(), StorageError> { unimplemented!() }
}
#[async_trait]
impl EventStorage for DummyPersistence {
    async fn create_event(&self, _event: &StoredWorkflowEvent) -> Result<i64, StorageError> { unimplemented!() }
    async fn get_event(&self, _id: i64) -> Result<Option<StoredWorkflowEvent>, StorageError> { unimplemented!() }
    async fn find_events_by_run_id(&self, _run_id: &str, _limit: i64, _offset: i64) -> Result<Vec<StoredWorkflowEvent>, StorageError> { unimplemented!() }
    async fn update_event(&self, _id: i64, _update: &UpdateStoredWorkflowEvent) -> Result<(), StorageError> { unimplemented!() }
    async fn archive_event(&self, _id: i64) -> Result<(), StorageError> { unimplemented!() }
    async fn delete_event(&self, _id: i64) -> Result<(), StorageError> { unimplemented!() }
    async fn delete_events_by_run_id(&self, _run_id: &str) -> Result<u64, StorageError> { unimplemented!() }
}
#[async_trait]
impl StateStorage for DummyPersistence {
    async fn create_state(&self, _state: &StoredWorkflowState) -> Result<(), StorageError> { unimplemented!() }
    async fn get_state(&self, _state_id: &str) -> Result<Option<StoredWorkflowState>, StorageError> { unimplemented!() }
    async fn find_states_by_run_id(&self, _run_id: &str, _limit: i64, _offset: i64) -> Result<Vec<StoredWorkflowState>, StorageError> { unimplemented!() }
    async fn update_state(&self, _state_id: &str, _changes: &UpdateStoredWorkflowState) -> Result<(), StorageError> { unimplemented!() }
    async fn delete_state(&self, _state_id: &str) -> Result<(), StorageError> { unimplemented!() }
}
#[async_trait]
impl ActivityStorage for DummyPersistence {
    async fn create_task(&self, _task: &StoredActivityTask) -> Result<(), StorageError> { unimplemented!() }
    async fn get_task(&self, _id: &str) -> Result<Option<StoredActivityTask>, StorageError> { unimplemented!() }
    async fn find_tasks_by_status(&self, _status: &str, _limit: i64, _offset: i64) -> Result<Vec<StoredActivityTask>, StorageError> { unimplemented!() }
    async fn update_task(&self, _id: &str, _update: &UpdateStoredActivityTask) -> Result<(), StorageError> { unimplemented!() }
    async fn delete_task(&self, _id: &str) -> Result<(), StorageError> { unimplemented!() }
}
#[async_trait]
impl TimerStorage for DummyPersistence {
    async fn create_timer(&self, _timer: &StoredTimer) -> Result<(), StorageError> { unimplemented!() }
    async fn get_timer(&self, _id: &str) -> Result<Option<StoredTimer>, StorageError> { unimplemented!() }
    async fn update_timer(&self, _id: &str, _update: &UpdateStoredTimer) -> Result<(), StorageError> { Ok(()) }
    async fn delete_timer(&self, _id: &str) -> Result<(), StorageError> { Ok(()) }
    async fn find_timers_before(&self, _before: NaiveDateTime, _limit: i64) -> Result<Vec<StoredTimer>, StorageError> { Ok(vec![]) }
}
#[async_trait]
impl TemplateStorage for DummyPersistence {
    async fn create_template(&self, _tpl: &StoredWorkflowTemplate) -> Result<(), StorageError> { unimplemented!() }
    async fn get_template(&self, _id: &str) -> Result<Option<StoredWorkflowTemplate>, StorageError> { unimplemented!() }
    async fn find_templates(&self, _limit: i64, _offset: i64) -> Result<Vec<StoredWorkflowTemplate>, StorageError> { unimplemented!() }
    async fn update_template(&self, _id: &str, _update: &UpdateStoredWorkflowTemplate) -> Result<(), StorageError> { unimplemented!() }
    async fn delete_template(&self, _id: &str) -> Result<(), StorageError> { unimplemented!() }
}
#[async_trait]
impl VisibilityStorage for DummyPersistence {
    async fn create_visibility(&self, _vis: &StoredWorkflowVisibility) -> Result<(), StorageError> { unimplemented!() }
    async fn get_visibility(&self, _id: &str) -> Result<Option<StoredWorkflowVisibility>, StorageError> { unimplemented!() }
    async fn find_visibilities_by_status(&self, _status: &str, _limit: i64, _offset: i64) -> Result<Vec<StoredWorkflowVisibility>, StorageError> { unimplemented!() }
    async fn update_visibility(&self, _id: &str, _update: &UpdateStoredWorkflowVisibility) -> Result<(), StorageError> { unimplemented!() }
    async fn delete_visibility(&self, _id: &str) -> Result<(), StorageError> { unimplemented!() }
}
#[async_trait]
impl QueueStorage for DummyPersistence {
    async fn create_queue_task(&self, _task: &StoredQueueTask) -> Result<(), StorageError> { unimplemented!() }
    async fn get_queue_task(&self, _id: &str) -> Result<Option<StoredQueueTask>, StorageError> { unimplemented!() }
    async fn update_queue_task(&self, _id: &str, _update: &UpdateStoredQueueTask) -> Result<(), StorageError> { unimplemented!() }
    async fn delete_queue_task(&self, _id: &str) -> Result<(), StorageError> { unimplemented!() }
    async fn find_queue_tasks_by_status(&self, _status: &str, _limit: i64, _offset: i64) -> Result<Vec<StoredQueueTask>, StorageError> { unimplemented!() }
    async fn find_queue_tasks_to_retry(&self, _before: NaiveDateTime, _limit: i64) -> Result<Vec<StoredQueueTask>, StorageError> { unimplemented!() }
}
#[async_trait]
impl TransactionManager for DummyPersistence {
    async fn begin_transaction(&self) -> Result<(), StorageError> { Ok(()) }
    async fn commit(&self) -> Result<(), StorageError> { Ok(()) }
    async fn rollback(&self) -> Result<(), StorageError> { Ok(()) }
} 
use sqlx::SqlitePool;
use chrono::NaiveDateTime;
use stepflow_storage::{
    error::StorageError,
    transaction::TransactionManager,
    entities::{
        workflow_execution::{StoredWorkflowExecution, UpdateStoredWorkflowExecution},
        workflow_event::{StoredWorkflowEvent, UpdateStoredWorkflowEvent},
        activity_task::{StoredActivityTask, UpdateStoredActivityTask},
        workflow_state::{StoredWorkflowState, UpdateStoredWorkflowState},
        timer::{StoredTimer, UpdateStoredTimer},
        workflow_template::{StoredWorkflowTemplate, UpdateStoredWorkflowTemplate},
        workflow_visibility::{StoredWorkflowVisibility, UpdateStoredWorkflowVisibility},
        queue_task::{StoredQueueTask, UpdateStoredQueueTask},
    },
};

use crate::persistence::{
    workflow_execution::WorkflowExecutionPersistence,
    workflow_event::WorkflowEventPersistence,
    activity_task::ActivityTaskPersistence,
    workflow_state::WorkflowStatePersistence,
    timer::TimerPersistence,
    workflow_template::WorkflowTemplatePersistence,
    workflow_visibility::WorkflowVisibilityPersistence,
    queue_task::QueueTaskPersistence,
};

pub struct SqliteStorageManager {
    pool: SqlitePool,
    workflow_execution: WorkflowExecutionPersistence,
    workflow_event: WorkflowEventPersistence,
    activity_task: ActivityTaskPersistence,
    workflow_state: WorkflowStatePersistence,
    timer: TimerPersistence,
    workflow_template: WorkflowTemplatePersistence,
    workflow_visibility: WorkflowVisibilityPersistence,
    queue_task: QueueTaskPersistence,
}

impl SqliteStorageManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            workflow_execution: WorkflowExecutionPersistence::new(pool.clone()),
            workflow_event: WorkflowEventPersistence::new(pool.clone()),
            activity_task: ActivityTaskPersistence::new(pool.clone()),
            workflow_state: WorkflowStatePersistence::new(pool.clone()),
            timer: TimerPersistence::new(pool.clone()),
            workflow_template: WorkflowTemplatePersistence::new(pool.clone()),
            workflow_visibility: WorkflowVisibilityPersistence::new(pool.clone()),
            queue_task: QueueTaskPersistence::new(pool.clone()),
            pool,
        }
    }

    /// 在事务中执行一个异步闭包
    pub async fn with_transaction<F, Fut, R>(&self, f: F) -> Result<R, StorageError>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<R, StorageError>> + Send,
        R: Send,
    {
        self.begin_transaction().await?;
        match f().await {
            Ok(result) => {
                self.commit().await?;
                Ok(result)
            }
            Err(e) => {
                self.rollback().await?;
                Err(e)
            }
        }
    }
}

#[async_trait::async_trait]
impl TransactionManager for SqliteStorageManager {
    async fn begin_transaction(&self) -> Result<(), StorageError> {
        sqlx::query("BEGIN TRANSACTION").execute(&self.pool).await.map_err(StorageError::from)?;
        Ok(())
    }

    async fn commit(&self) -> Result<(), StorageError> {
        sqlx::query("COMMIT").execute(&self.pool).await.map_err(StorageError::from)?;
        Ok(())
    }

    async fn rollback(&self) -> Result<(), StorageError> {
        sqlx::query("ROLLBACK").execute(&self.pool).await.map_err(StorageError::from)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl stepflow_storage::traits::WorkflowStorage for SqliteStorageManager {
    async fn create_execution(&self, exec: &StoredWorkflowExecution) -> Result<(), StorageError> {
        self.workflow_execution.create_execution(exec).await
    }

    async fn get_execution(&self, run_id: &str) -> Result<Option<StoredWorkflowExecution>, StorageError> {
        self.workflow_execution.get_execution(run_id).await
    }

    async fn find_executions(&self, limit: i64, offset: i64) -> Result<Vec<StoredWorkflowExecution>, StorageError> {
        self.workflow_execution.find_executions(limit, offset).await
    }

    async fn find_executions_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<StoredWorkflowExecution>, StorageError> {
        self.workflow_execution.find_executions_by_status(status, limit, offset).await
    }

    async fn update_execution(&self, run_id: &str, changes: &UpdateStoredWorkflowExecution) -> Result<(), StorageError> {
        self.workflow_execution.update_execution(run_id, changes).await
    }

    async fn delete_execution(&self, run_id: &str) -> Result<(), StorageError> {
        self.workflow_execution.delete_execution(run_id).await
    }
}

#[async_trait::async_trait]
impl stepflow_storage::traits::EventStorage for SqliteStorageManager {
    async fn create_event(&self, event: &StoredWorkflowEvent) -> Result<i64, StorageError> {
        self.workflow_event.create_event(event).await
    }

    async fn get_event(&self, id: i64) -> Result<Option<StoredWorkflowEvent>, StorageError> {
        self.workflow_event.get_event(id).await
    }

    async fn find_events_by_run_id(&self, run_id: &str, limit: i64, offset: i64) -> Result<Vec<StoredWorkflowEvent>, StorageError> {
        self.workflow_event.find_events_by_run_id(run_id, limit, offset).await
    }

    async fn update_event(&self, id: i64, changes: &UpdateStoredWorkflowEvent) -> Result<(), StorageError> {
        self.workflow_event.update_event(id, changes).await
    }

    async fn archive_event(&self, id: i64) -> Result<(), StorageError> {
        self.workflow_event.archive_event(id).await
    }

    async fn delete_event(&self, id: i64) -> Result<(), StorageError> {
        self.workflow_event.delete_event(id).await
    }

    async fn delete_events_by_run_id(&self, run_id: &str) -> Result<u64, StorageError> {
        self.workflow_event.delete_events_by_run_id(run_id).await
    }
}

#[async_trait::async_trait]
impl stepflow_storage::traits::ActivityStorage for SqliteStorageManager {
    async fn create_task(&self, task: &StoredActivityTask) -> Result<(), StorageError> {
        self.activity_task.create_task(task).await
    }

    async fn get_task(&self, task_token: &str) -> Result<Option<StoredActivityTask>, StorageError> {
        self.activity_task.get_task(task_token).await
    }

    async fn find_tasks_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<StoredActivityTask>, StorageError> {
        self.activity_task.find_tasks_by_status(status, limit, offset).await
    }

    async fn update_task(&self, task_token: &str, changes: &UpdateStoredActivityTask) -> Result<(), StorageError> {
        self.activity_task.update_task(task_token, changes).await
    }

    async fn delete_task(&self, task_token: &str) -> Result<(), StorageError> {
        self.activity_task.delete_task(task_token).await
    }
}

#[async_trait::async_trait]
impl stepflow_storage::traits::StateStorage for SqliteStorageManager {
    async fn create_state(&self, state: &StoredWorkflowState) -> Result<(), StorageError> {
        self.workflow_state.create_state(state).await
    }

    async fn get_state(&self, state_id: &str) -> Result<Option<StoredWorkflowState>, StorageError> {
        self.workflow_state.get_state(state_id).await
    }

    async fn find_states_by_run_id(&self, run_id: &str, limit: i64, offset: i64) -> Result<Vec<StoredWorkflowState>, StorageError> {
        self.workflow_state.find_states_by_run_id(run_id, limit, offset).await
    }

    async fn update_state(&self, state_id: &str, changes: &UpdateStoredWorkflowState) -> Result<(), StorageError> {
        self.workflow_state.update_state(state_id, changes).await
    }

    async fn delete_state(&self, state_id: &str) -> Result<(), StorageError> {
        self.workflow_state.delete_state(state_id).await
    }
}

#[async_trait::async_trait]
impl stepflow_storage::traits::TimerStorage for SqliteStorageManager {
    async fn create_timer(&self, timer: &StoredTimer) -> Result<(), StorageError> {
        self.timer.create_timer(timer).await
    }

    async fn get_timer(&self, timer_id: &str) -> Result<Option<StoredTimer>, StorageError> {
        self.timer.get_timer(timer_id).await
    }

    async fn update_timer(&self, timer_id: &str, changes: &UpdateStoredTimer) -> Result<(), StorageError> {
        self.timer.update_timer(timer_id, changes).await
    }

    async fn delete_timer(&self, timer_id: &str) -> Result<(), StorageError> {
        self.timer.delete_timer(timer_id).await
    }

    async fn find_timers_before(&self, before: NaiveDateTime, limit: i64) -> Result<Vec<StoredTimer>, StorageError> {
        self.timer.find_timers_before(before, limit).await
    }
}

#[async_trait::async_trait]
impl stepflow_storage::traits::TemplateStorage for SqliteStorageManager {
    async fn create_template(&self, tpl: &StoredWorkflowTemplate) -> Result<(), StorageError> {
        self.workflow_template.create_template(tpl).await
    }

    async fn get_template(&self, template_id: &str) -> Result<Option<StoredWorkflowTemplate>, StorageError> {
        self.workflow_template.get_template(template_id).await
    }

    async fn find_templates(&self, limit: i64, offset: i64) -> Result<Vec<StoredWorkflowTemplate>, StorageError> {
        self.workflow_template.find_templates(limit, offset).await
    }

    async fn update_template(&self, template_id: &str, changes: &UpdateStoredWorkflowTemplate) -> Result<(), StorageError> {
        self.workflow_template.update_template(template_id, changes).await
    }

    async fn delete_template(&self, template_id: &str) -> Result<(), StorageError> {
        self.workflow_template.delete_template(template_id).await
    }
}

#[async_trait::async_trait]
impl stepflow_storage::traits::VisibilityStorage for SqliteStorageManager {
    async fn create_visibility(&self, vis: &StoredWorkflowVisibility) -> Result<(), StorageError> {
        self.workflow_visibility.create_visibility(vis).await
    }

    async fn get_visibility(&self, run_id: &str) -> Result<Option<StoredWorkflowVisibility>, StorageError> {
        self.workflow_visibility.get_visibility(run_id).await
    }

    async fn find_visibilities_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<StoredWorkflowVisibility>, StorageError> {
        self.workflow_visibility.find_visibilities_by_status(status, limit, offset).await
    }

    async fn update_visibility(&self, run_id: &str, changes: &UpdateStoredWorkflowVisibility) -> Result<(), StorageError> {
        self.workflow_visibility.update_visibility(run_id, changes).await
    }

    async fn delete_visibility(&self, run_id: &str) -> Result<(), StorageError> {
        self.workflow_visibility.delete_visibility(run_id).await
    }
}

#[async_trait::async_trait]
impl stepflow_storage::traits::QueueStorage for SqliteStorageManager {
    async fn create_queue_task(&self, task: &StoredQueueTask) -> Result<(), StorageError> {
        self.queue_task.create_queue_task(task).await
    }

    async fn get_queue_task(&self, task_id: &str) -> Result<Option<StoredQueueTask>, StorageError> {
        self.queue_task.get_queue_task(task_id).await
    }

    async fn update_queue_task(&self, task_id: &str, changes: &UpdateStoredQueueTask) -> Result<(), StorageError> {
        self.queue_task.update_queue_task(task_id, changes).await
    }

    async fn delete_queue_task(&self, task_id: &str) -> Result<(), StorageError> {
        self.queue_task.delete_queue_task(task_id).await
    }

    async fn find_queue_tasks_by_status(&self, status: &str, limit: i64, offset: i64) -> Result<Vec<StoredQueueTask>, StorageError> {
        self.queue_task.find_queue_tasks_by_status(status, limit, offset).await
    }

    async fn find_queue_tasks_to_retry(&self, before: NaiveDateTime, limit: i64) -> Result<Vec<StoredQueueTask>, StorageError> {
        self.queue_task.find_queue_tasks_to_retry(before, limit).await
    }
} 
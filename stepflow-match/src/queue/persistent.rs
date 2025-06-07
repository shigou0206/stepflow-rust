use chrono::Utc;       
use serde_json::Value;
use std::sync::Arc;
use std::fmt;
use thiserror::Error;
use uuid::Uuid;
use stepflow_storage::persistence_manager::PersistenceManager;
use stepflow_storage::entities::queue_task::{StoredQueueTask, UpdateStoredQueueTask};
use async_trait::async_trait;

use super::traits::TaskStore;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Task not found: {run_id}/{state_name}")]
    TaskNotFound { run_id: String, state_name: String },
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Invalid status transition from {from} to {to}")]
    InvalidStatusTransition { from: TaskStatus, to: TaskStatus },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retrying,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TaskStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Retrying => "retrying",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "processing" => Some(Self::Processing),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "retrying" => Some(Self::Retrying),
            _ => None,
        }
    }

    fn is_valid_transition(&self, to: TaskStatus) -> bool {
        use TaskStatus::*;
        matches!(
            (self, to),
            (Pending, Processing) |
            (Processing, Completed) |
            (Processing, Failed) |
            (Failed, Retrying) |
            (Retrying, Processing)
        )
    }
}

pub struct PersistentStore {
    persistence: Arc<dyn PersistenceManager>,
}

impl PersistentStore {
    pub fn new(persistence: Arc<dyn PersistenceManager>) -> Self {
        Self { persistence }
    }

    pub async fn find_task(&self, run_id: &str, state_name: &str) -> Result<StoredQueueTask, StoreError> {
        let tasks = self.persistence.find_queue_tasks_by_status("", 1, 0)
            .await
            .map_err(|e| StoreError::StorageError(e.to_string()))?;

        tasks.into_iter()
            .find(|t| t.run_id == run_id && t.state_name == state_name)
            .ok_or_else(|| StoreError::TaskNotFound {
                run_id: run_id.to_string(),
                state_name: state_name.to_string(),
            })
    }
}

#[async_trait]
impl TaskStore for PersistentStore {
    async fn insert_task(
        &self,
        persistence: &Arc<dyn PersistenceManager>,
        run_id: &str,
        state_name: &str,
        resource: &str, 
        input: &Value,
    ) -> Result<(), String> {
        let now = Utc::now().naive_utc();
        let task = StoredQueueTask {
            task_id: Uuid::new_v4().to_string(),
            run_id: run_id.to_string(),
            state_name: state_name.to_string(),
            resource: resource.to_string(),
            task_payload: Some(input.clone()),
            status: TaskStatus::Pending.as_str().to_string(),
            attempts: 0,
            max_attempts: 3, 
            priority: Some(0),
            timeout_seconds: Some(300),
            error_message: None,
            last_error_at: None,
            next_retry_at: None,
            queued_at: now,
            processing_at: None,
            completed_at: None,
            failed_at: None,
            created_at: now,
            updated_at: now,
        };

        persistence.create_queue_task(&task)
            .await
            .map_err(|e| e.to_string())
    }

    async fn update_task_status(
        &self,
        persistence: &Arc<dyn PersistenceManager>,
        run_id: &str,
        state_name: &str,
        status: &str,
        result: &Value,
    ) -> Result<(), String> {
        let task = self.find_task(run_id, state_name)
            .await
            .map_err(|e| e.to_string())?;

        let now = Utc::now().naive_utc();
        let new_status = TaskStatus::from_str(status)
            .ok_or_else(|| format!("Invalid status: {}", status))?;
        let current_status = TaskStatus::from_str(&task.status)
            .ok_or_else(|| format!("Invalid current status: {}", task.status))?;

        if !current_status.is_valid_transition(new_status) {
            return Err(StoreError::InvalidStatusTransition {
                from: current_status,
                to: new_status,
            }.to_string());
        }

        let mut changes = UpdateStoredQueueTask {
            status: Some(status.to_string()),
            task_payload: Some(Some(result.clone())),
            attempts: None,
            priority: Some(0),
            timeout_seconds: Some(300),
            error_message: None,
            last_error_at: None,
            next_retry_at: None,
            processing_at: None,
            completed_at: None,
            failed_at: None,
        };

        match new_status {
            TaskStatus::Completed => {
                changes.completed_at = Some(Some(now));
            }
            TaskStatus::Failed => {
                let attempts = task.attempts + 1;
                changes.attempts = Some(attempts);
                changes.failed_at = Some(Some(now));
                changes.error_message = Some(Some(result.to_string()));
            }
            TaskStatus::Processing => {
                changes.processing_at = Some(Some(now));
            }
            TaskStatus::Retrying => {
                let attempts = task.attempts + 1;
                changes.attempts = Some(attempts);
                changes.error_message = Some(Some(result.to_string()));
                changes.last_error_at = Some(Some(now));
                changes.next_retry_at = Some(Some(now + chrono::Duration::seconds(30))); // 30秒后重试
            }
            _ => {}
        }

        persistence.update_queue_task(&task.task_id, &changes)
            .await
            .map_err(|e| e.to_string())
    }
} 
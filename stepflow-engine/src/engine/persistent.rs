use chrono::{NaiveDateTime, Utc, Duration};
use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use std::sync::Arc;
use std::fmt;
use thiserror::Error;
use uuid::Uuid;
use stepflow_storage::persistence_manager::PersistenceManager;
use stepflow_sqlite::models::queue_task::{QueueTask, UpdateQueueTask};

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

pub struct RetryPolicy {
    pub max_attempts: i64,
    pub base_delay: i64,    // 基础延迟（秒）
    pub max_delay: i64,     // 最大延迟（秒）
    pub multiplier: f64,    // 延迟增长倍数
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: 5,
            max_delay: 30,
            multiplier: 2.0,
        }
    }
}

pub struct PersistentStore {
    persistence: Arc<dyn PersistenceManager>,
    retry_policy: RetryPolicy,
}

impl PersistentStore {
    pub fn new(persistence: Arc<dyn PersistenceManager>) -> Self {
        Self { 
            persistence,
            retry_policy: RetryPolicy::default(),
        }
    }

    pub fn with_retry_policy(persistence: Arc<dyn PersistenceManager>, retry_policy: RetryPolicy) -> Self {
        Self { persistence, retry_policy }
    }

    async fn find_task(&self, run_id: &str, state_name: &str) -> Result<QueueTask, StoreError> {
        let tasks = self.persistence
            .find_queue_tasks_by_status("pending", 1, 0)
            .await
            .map_err(|e| StoreError::StorageError(e.to_string()))?;
        
        tasks.into_iter()
            .find(|t| t.run_id == run_id && t.state_name == state_name)
            .ok_or_else(|| StoreError::TaskNotFound { 
                run_id: run_id.to_string(), 
                state_name: state_name.to_string() 
            })
    }

    fn calculate_next_retry(&self, attempts: i64, now: NaiveDateTime) -> NaiveDateTime {
        let delay = (self.retry_policy.base_delay as f64 * 
            self.retry_policy.multiplier.powi(attempts as i32)) as i64;
        let delay = delay.min(self.retry_policy.max_delay);
        now + Duration::seconds(delay)
    }
}

#[async_trait::async_trait]
impl TaskStore for PersistentStore {
    async fn insert_task(
        &self,
        _tx: &mut Transaction<'_, Sqlite>,
        run_id: &str,
        state_name: &str,
        _resource: &str,
        input: &Value,
    ) -> Result<(), String> {
        let now = Utc::now().naive_utc();
        let task = QueueTask {
            task_id: Uuid::new_v4().to_string(),
            run_id: run_id.to_string(),
            state_name: state_name.to_string(),
            task_payload: Some(input.to_string()),
            status: TaskStatus::Pending.as_str().to_string(),
            attempts: 0,
            max_attempts: self.retry_policy.max_attempts,
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

        self.persistence.create_queue_task(&task)
            .await
            .map_err(|e| e.to_string())
    }

    async fn update_task_status(
        &self,
        _tx: &mut Transaction<'_, Sqlite>,
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

        let changes = match new_status {
            TaskStatus::Completed => UpdateQueueTask {
                status: Some(new_status.as_str().to_string()),
                completed_at: Some(Some(now)),
                updated_at: Some(now),
                ..Default::default()
            },
            TaskStatus::Failed => {
                let attempts = task.attempts + 1;
                if attempts >= self.retry_policy.max_attempts {
                    UpdateQueueTask {
                        status: Some(TaskStatus::Failed.as_str().to_string()),
                        attempts: Some(attempts),
                        failed_at: Some(Some(now)),
                        error_message: Some(Some(result.to_string())),
                        updated_at: Some(Some(now)),
                        ..Default::default()
                    }
                } else {
                    let next_retry = self.calculate_next_retry(attempts, now);
                    UpdateQueueTask {
                        status: Some(TaskStatus::Retrying.as_str().to_string()),
                        attempts: Some(attempts),
                        error_message: Some(Some(result.to_string())),
                        last_error_at: Some(Some(now)),
                        next_retry_at: Some(Some(next_retry)),
                        updated_at: Some(Some(now)),
                        ..Default::default()
                    }
                }
            },
            _ => UpdateQueueTask {
                status: Some(new_status.as_str().to_string()),
                updated_at: Some(now),
                ..Default::default()
            }
        };

        self.persistence.update_queue_task(&task.task_id, &changes)
            .await
            .map_err(|e| e.to_string())
    }
} 
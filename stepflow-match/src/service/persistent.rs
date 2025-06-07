use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use serde_json::Value;
use std::any::Any;
use stepflow_storage::persistence_manager::PersistenceManager;
use crate::service::MatchService;
use crate::queue::PersistentStore;
use crate::queue::TaskStore;
use stepflow_dto::dto::match_stats::MatchStats;
use stepflow_dto::dto::queue_task::QueueTaskDto;

/// 将 PersistentStore 包装为 MatchService 的适配器
pub struct PersistentMatchService {
    store: Arc<PersistentStore>,
    persistence: Arc<dyn PersistenceManager>,
}

impl PersistentMatchService {
    pub fn new(store: Arc<PersistentStore>, persistence: Arc<dyn PersistenceManager>) -> Arc<Self> {
        Arc::new(Self { store, persistence })
    }
}

#[async_trait]
impl MatchService for PersistentMatchService {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn queue_stats(&self) -> Vec<MatchStats> {
        // 数据库版实现暂无准确排队数据，这里返回空或占位值
        vec![MatchStats {
            queue: "default_task_queue".to_string(),
            pending_tasks: 0,
            waiting_workers: 0,
        }]
    }

    async fn poll_task(&self, _queue: &str, _worker_id: &str, _timeout: Duration) -> Option<QueueTaskDto> {
        let tasks = self
            .persistence
            .find_queue_tasks_by_status("pending", 1, 0)
            .await
            .ok()?;

        if let Some(task) = tasks.into_iter().next() {
            // 更新状态为 processing（可选）
            let _ = self.persistence.update_queue_task(
                &task.task_id,
                &stepflow_storage::entities::queue_task::UpdateStoredQueueTask {
                    status: Some("processing".to_string()),
                    processing_at: Some(Some(chrono::Utc::now().naive_utc())),
                    ..Default::default()
                },
            ).await;

            return Some(QueueTaskDto {
                task_id: task.task_id.into(),
                run_id: task.run_id,
                state_name: task.state_name,
                resource: task.resource,
                task_payload: task.task_payload,
                status: task.status,
                attempts: task.attempts,
                max_attempts: task.max_attempts,
                priority: task.priority.map(|p| p as u8),
                timeout_seconds: task.timeout_seconds,
                error_message: task.error_message,
                last_error_at: task.last_error_at.map(|dt| dt.and_utc()),
                next_retry_at: task.next_retry_at.map(|dt| dt.and_utc()),
                queued_at: task.queued_at.and_utc(),
                processing_at: task.processing_at.map(|dt| dt.and_utc()),
                completed_at: task.completed_at.map(|dt| dt.and_utc()),
                failed_at: task.failed_at.map(|dt| dt.and_utc()),
            });
        }

        None
    }

    async fn enqueue_task(&self, _queue: &str, task: QueueTaskDto) -> Result<(), String> {
        self.store
            .insert_task(
                &self.persistence,
                &task.run_id,
                &task.state_name,
                &task.resource,
                &task.task_payload.clone().unwrap_or(Value::Null),
            )
            .await
    }

    async fn wait_for_completion(
        &self,
        _run_id: &str,
        _state_name: &str,
        input: &Value,
        _persistence: Arc<dyn PersistenceManager>,
    ) -> Result<Value, String> {
        Ok(input.clone())
    }
}
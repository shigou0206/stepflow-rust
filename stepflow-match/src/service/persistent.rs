use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use serde_json::Value;
use std::any::Any;
use stepflow_storage::persistence_manager::PersistenceManager;
use crate::service::{MatchService, Task};
use crate::queue::PersistentStore;
use crate::queue::TaskStore;
use crate::service::interface::MatchStats;

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

    async fn poll_task(&self, _queue: &str, _worker_id: &str, _timeout: Duration) -> Option<Task> {
        let tasks = self
            .persistence
            .find_queue_tasks_by_status("pending", 1, 0)
            .await
            .ok()?;
    
        if let Some(task) = tasks.into_iter().next() {
            // 更新为 processing（可选）
            let _ = self.persistence.update_queue_task(
                &task.task_id,
                &stepflow_storage::entities::queue_task::UpdateStoredQueueTask {
                    status: Some("processing".to_string()),
                    processing_at: Some(Some(chrono::Utc::now().naive_utc())),
                    ..Default::default()
                },
            ).await;
    
            return Some(Task {
                run_id: task.run_id,
                state_name: task.state_name.unwrap_or_default(),
                input: task.task_payload,
                task_type: task.task_type,
                task_token: Some(task.task_id),
                priority: Some(task.priority.unwrap_or(0) as u8),
                attempt: Some(task.attempts),
                max_attempts: Some(task.max_attempts),
                timeout_seconds: task.timeout_seconds,
                scheduled_at: Some(task.queued_at),
            });
        }
    
        None
    }

    async fn enqueue_task(&self, _queue: &str, task: Task) -> Result<(), String> {
        // 使用 store.insert_task 创建数据库任务记录
        self.store
            .insert_task(
                &self.persistence,
                &task.run_id,
                &task.state_name,
                &task.task_type,
                &task.input.clone().unwrap_or(Value::Null),
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
        // 简单实现：直接返回输入
        Ok(input.clone())
    }
}
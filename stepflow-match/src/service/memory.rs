use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;
use async_trait::async_trait;
use serde_json::Value;
use std::any::Any;

use stepflow_storage::persistence_manager::PersistenceManager;
use crate::service::interface::{MatchService};
use stepflow_dto::dto::match_stats::MatchStats;
use stepflow_dto::dto::queue_task::QueueTaskDto;

/// 内存版的任务匹配服务实现
pub struct MemoryMatchService {
    // 工作进程ID -> 等待任务的工作进程
    waiting_workers: Mutex<HashMap<String, tokio::sync::oneshot::Sender<QueueTaskDto>>>,
    // 队列名称 -> 待处理的任务队列
    pending_tasks: Mutex<HashMap<String, VecDeque<QueueTaskDto>>>,
}

impl MemoryMatchService {
    /// 创建新的内存匹配服务实例
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            waiting_workers: Mutex::new(HashMap::new()),
            pending_tasks: Mutex::new(HashMap::new()),
        })
    }

    /// 获取指定队列中等待的任务数量
    pub async fn pending_tasks_count(&self, queue: &str) -> usize {
        self.pending_tasks
            .lock()
            .await
            .get(queue)
            .map(|q| q.len())
            .unwrap_or(0)
    }

    /// 获取指定队列中等待的 worker 数量
    pub async fn waiting_workers_count(&self, _queue: &str) -> usize {
        self.waiting_workers.lock().await.len()
    }
}

#[async_trait]
impl MatchService for MemoryMatchService {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn queue_stats(&self) -> Vec<MatchStats> {
        let pending = self.pending_tasks.lock().await;
        let waiting = self.waiting_workers.lock().await;
        pending
            .iter()
            .map(|(queue, tasks)| MatchStats {
                queue: queue.clone(),
                pending_tasks: tasks.len(),
                waiting_workers: waiting.len(), // 简化实现
            })
            .collect()
    }

    async fn poll_task(&self, queue: &str, worker_id: &str, wait_time: Duration) -> Option<QueueTaskDto> {
        let mut pending_tasks = self.pending_tasks.lock().await;
        if let Some(tasks) = pending_tasks.get_mut(queue) {
            if let Some(task) = tasks.pop_front() {
                return Some(task);
            }
        }

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.waiting_workers.lock().await.insert(worker_id.to_string(), tx);

        match timeout(wait_time, rx).await {
            Ok(Ok(task)) => Some(task),
            _ => {
                self.waiting_workers.lock().await.remove(worker_id);
                None
            }
        }
    }

    async fn enqueue_task(&self, queue: &str, task: QueueTaskDto) -> Result<(), String> {
        let mut waiting = self.waiting_workers.lock().await;
        if let Some((_, waiter)) = waiting.drain().next() {
            let _ = waiter.send(task);
            return Ok(());
        }

        let mut pending = self.pending_tasks.lock().await;
        pending.entry(queue.to_string()).or_default().push_back(task);
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;
    use serde_json::json;
    use chrono::Utc;
    use stepflow_storage::mock_persistence::DummyPersistence;

    fn sample_task() -> QueueTaskDto {
        QueueTaskDto {
            task_id: "task1".to_string(),
            run_id: "run1".to_string(),
            state_name: "stateA".to_string(),
            resource: "resource1".to_string(),
            task_payload: Some(json!({"key": "value"})),
            status: "PENDING".to_string(),
            attempts: 1,
            max_attempts: 3,
            error_message: None,
            last_error_at: None,
            next_retry_at: None,
            queued_at: Utc::now(),
            processing_at: None,
            completed_at: None,
            failed_at: None,
        }
    }

    #[tokio::test]
    async fn test_pending_tasks_count() {
        let service = MemoryMatchService::new();
        assert_eq!(service.pending_tasks_count("q1").await, 0);
    }

    #[tokio::test]
    async fn test_waiting_workers_count() {
        let service = MemoryMatchService::new();
        assert_eq!(service.waiting_workers_count("q1").await, 0);
    }

    #[tokio::test]
    async fn test_enqueue_and_poll_task() {
        let service = MemoryMatchService::new();
        let task = sample_task();
        service.enqueue_task("q1", task.clone()).await.unwrap();

        let result = service.poll_task("q1", "worker1", Duration::from_secs(1)).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().task_id, task.task_id);
    }

    #[tokio::test]
    async fn test_wait_for_completion() {
        let service = MemoryMatchService::new();
        let input = json!({"input": 123});
        let result = service.wait_for_completion("run", "state", &input, Arc::new(DummyPersistence)).await;
        assert_eq!(result.unwrap(), input);
    }
}
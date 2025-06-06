use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;
use async_trait::async_trait;
use serde_json::Value;
use stepflow_storage::persistence_manager::PersistenceManager;

use super::interface::{MatchService, Task};

/// 内存版的任务匹配服务实现
pub struct MemoryMatchService {
    // 工作进程ID -> 等待任务的工作进程
    waiting_workers: Mutex<HashMap<String, tokio::sync::oneshot::Sender<Task>>>,
    // 队列名称 -> 待处理的任务队列
    pending_tasks: Mutex<HashMap<String, VecDeque<Task>>>,
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
        self.waiting_workers
            .lock()
            .await
            .values()
            .count()
    }
}

#[async_trait]
impl MatchService for MemoryMatchService {
    async fn poll_task(&self, queue: &str, worker_id: &str, wait_time: Duration) -> Option<Task> {
        let mut pending_tasks = self.pending_tasks.lock().await;
        
        // 检查队列中是否有待处理的任务
        if let Some(tasks) = pending_tasks.get_mut(queue) {
            if let Some(task) = tasks.pop_front() {
                return Some(task);
            }
        }

        // 如果没有任务，则等待新任务
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        // 注册等待的工作进程
        self.waiting_workers.lock().await.insert(worker_id.to_string(), tx);
        
        // 等待新任务或超时
        match timeout(wait_time, rx).await {
            Ok(Ok(task)) => Some(task),
            _ => {
                // 超时或通道关闭，移除等待的工作进程
                self.waiting_workers.lock().await.remove(worker_id);
                None
            }
        }
    }

    async fn enqueue_task(&self, queue: &str, task: Task) -> Result<(), String> {
        let mut waiting_workers = self.waiting_workers.lock().await;
        
        // 检查是否有等待的工作进程
        if let Some((_, worker)) = waiting_workers.drain().next() {
            // 如果有等待的工作进程，直接发送任务
            let _ = worker.send(task);
            return Ok(());
        }
        
        // 如果没有等待的工作进程，将任务加入队列
        let mut pending_tasks = self.pending_tasks.lock().await;
        let tasks = pending_tasks.entry(queue.to_string()).or_insert_with(VecDeque::new);
        tasks.push_back(task);
        
        Ok(())
    }

    async fn wait_for_completion(
        &self,
        _run_id: &str,
        _state_name: &str,
        input: &Value,
        _persistence: Arc<dyn PersistenceManager>,
    ) -> Result<Value, String> {
        // 简单实现：直接返回输入值
        Ok(input.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;
    use serde_json::json;
    use stepflow_storage::mock_persistence::DummyPersistence;

    #[tokio::test]
    async fn test_pending_tasks_count() {
        let service = MemoryMatchService::new();
        assert_eq!(service.pending_tasks_count("test_queue").await, 0);
    }

    #[tokio::test]
    async fn test_waiting_workers_count() {
        let service = MemoryMatchService::new();
        assert_eq!(service.waiting_workers_count("test_queue").await, 0);
    }

    #[tokio::test]
    async fn test_enqueue_and_poll_task() {
        let service = MemoryMatchService::new();
        let task = Task {
            run_id: "run1".to_string(),
            state_name: "stateA".to_string(),
            input: Some(serde_json::json!({})),
            task_type: "typeA".to_string(),
            task_token: Some("token1".to_string()),
            priority: Some(1),
            attempt: Some(1),
            max_attempts: Some(3),
            timeout_seconds: Some(60),
            scheduled_at: Some(chrono::NaiveDateTime::from_timestamp(0, 0)), 
        };
        service.enqueue_task("test_queue", task.clone()).await.unwrap();
        let polled_task = service.poll_task("test_queue", "worker_1", Duration::from_secs(1)).await;
        assert!(polled_task.is_some());
    }

    #[tokio::test]
    async fn test_wait_for_completion() {
        let service = MemoryMatchService::new();
        let input = json!({"key": "value"});
        let result = service.wait_for_completion("run_id", "state_name", &input, Arc::new(DummyPersistence)).await;
        assert_eq!(result.unwrap(), input);
    }
}
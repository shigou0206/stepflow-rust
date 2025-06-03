use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, oneshot};
use tokio::time::timeout;
use async_trait::async_trait;

use super::interface::{MatchService, Task};

/// 内存版的任务匹配服务实现
pub struct MemoryMatchService {
    // 队列名称 -> 等待该队列的 workers (oneshot channel 发送端)
    waiting_workers: Mutex<HashMap<String, Vec<oneshot::Sender<Task>>>>,
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
    pub async fn waiting_workers_count(&self, queue: &str) -> usize {
        self.waiting_workers
            .lock()
            .await
            .get(queue)
            .map(|w| w.len())
            .unwrap_or(0)
    }
}

#[async_trait]
impl MatchService for MemoryMatchService {
    async fn poll_task(&self, queue: &str, _worker_id: &str, timeout_duration: Duration) -> Option<Task> {
        // 1. 先检查是否有待处理的任务
        {
            let mut pending = self.pending_tasks.lock().await;
            if let Some(queue_tasks) = pending.get_mut(queue) {
                if let Some(task) = queue_tasks.pop_front() {
                    return Some(task);
                }
            }
        }

        // 2. 如果没有任务，创建一个 channel 并等待
        let (tx, rx) = oneshot::channel();
        
        // 将 worker 加入等待列表
        {
            let mut workers = self.waiting_workers.lock().await;
            workers.entry(queue.to_string())
                .or_insert_with(Vec::new)
                .push(tx);
        }

        // 3. 等待任务或超时
        match timeout(timeout_duration, rx).await {
            Ok(Ok(task)) => Some(task),
            _ => None
        }
    }

    async fn enqueue_task(&self, queue: &str, task: Task) -> Result<(), String> {
        // 1. 检查是否有等待的 worker
        let worker_tx = {
            let mut workers = self.waiting_workers.lock().await;
            if let Some(queue_workers) = workers.get_mut(queue) {
                queue_workers.pop()
            } else {
                None
            }
        };

        // 2. 如果有等待的 worker，直接发送任务
        if let Some(tx) = worker_tx {
            let _ = tx.send(task);
            return Ok(());
        }

        // 3. 如果没有等待的 worker，将任务加入待处理队列
        let mut pending = self.pending_tasks.lock().await;
        let queue_tasks = pending
            .entry(queue.to_string())
            .or_insert_with(VecDeque::new);
        
        queue_tasks.push_back(task);
        Ok(())
    }
} 
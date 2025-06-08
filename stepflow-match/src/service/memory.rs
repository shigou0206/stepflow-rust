// memory.rs  —— 经过 DynPM 适配后的最终版
use std::{
    any::Any,
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use serde_json::Value;
use tokio::{
    sync::{Mutex, oneshot},
    time::timeout,
};

use crate::{
    service::interface::{MatchService, DynPM},   // ← 引入 DynPM
};
use stepflow_dto::{
    dto::match_stats::MatchStats,
    dto::queue_task::QueueTaskDto,
};

/// 内存版任务匹配服务
pub struct MemoryMatchService {
    /// 队列名 → 待处理任务
    pending_tasks:  Mutex<HashMap<String, VecDeque<QueueTaskDto>>>,
    /// worker_id → 等待中的 oneshot Sender
    waiting_workers: Mutex<HashMap<String, oneshot::Sender<QueueTaskDto>>>,
}

impl MemoryMatchService {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            pending_tasks:  Mutex::new(HashMap::new()),
            waiting_workers: Mutex::new(HashMap::new()),
        })
    }

    // 仅测试用的辅助函数
    pub async fn pending_tasks_count(&self, queue: &str) -> usize {
        self.pending_tasks
            .lock()
            .await
            .get(queue)
            .map_or(0, |q| q.len())
    }
    pub async fn waiting_workers_count(&self) -> usize {
        self.waiting_workers.lock().await.len()
    }
}

#[async_trait]
impl MatchService for MemoryMatchService {
    fn as_any(&self) -> &dyn Any { self }

    // 1. 队列统计
    async fn queue_stats(&self) -> Vec<MatchStats> {
        let pending  = self.pending_tasks.lock().await;
        let waiting  = self.waiting_workers.lock().await;
        pending
            .iter()
            .map(|(queue, tasks)| MatchStats {
                queue: queue.clone(),
                pending_tasks: tasks.len(),
                waiting_workers: waiting.len(),   // demo：全部 worker 共享
            })
            .collect()
    }

    // 2. worker 轮询
    async fn poll_task(
        &self,
        queue: &str,
        worker_id: &str,
        wait_time: Duration,
    ) -> Option<QueueTaskDto> {
        // ① 先看是否已有任务排队
        if let Some(task) = self.pending_tasks
            .lock()
            .await
            .get_mut(queue)
            .and_then(|q| q.pop_front())
        {
            return Some(task);
        }

        // ② 否则挂起等待
        let (tx, rx) = oneshot::channel();
        self.waiting_workers
            .lock()
            .await
            .insert(worker_id.to_string(), tx);

        match timeout(wait_time, rx).await {
            Ok(Ok(task)) => Some(task),
            _ => {
                // 超时 / 发送端已取消
                self.waiting_workers.lock().await.remove(worker_id);
                None
            }
        }
    }

    // 3. 入队
    async fn enqueue_task(&self, queue: &str, task: QueueTaskDto) -> Result<(), String> {
        // 有等待的 worker？直接派发
        if let Some((_, waiter)) = self.waiting_workers.lock().await.drain().next() {
            let _ = waiter.send(task);
            return Ok(());
        }
        // 否则进入 pending 队列
        self.pending_tasks
            .lock()
            .await
            .entry(queue.to_string())
            .or_default()
            .push_back(task);
        Ok(())
    }

    // 4. 等待完成（内存实现：直接回显）
    async fn wait_for_completion(
        &self,
        _run_id: &str,
        _state_name: &str,
        input: &Value,
        _pm: &DynPM,                    // ⚠️ 必须与 trait 保持一致
    ) -> Result<Value, String> {
        Ok(input.clone())
    }
}

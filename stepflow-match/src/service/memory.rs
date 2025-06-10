// src/service/memory_match_service.rs

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

use crate::service::interface::{MatchService, DynPM};
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
    /// 构造
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            pending_tasks:  Mutex::new(HashMap::new()),
            waiting_workers: Mutex::new(HashMap::new()),
        })
    }
}

#[async_trait]
impl MatchService for MemoryMatchService {
    fn as_any(&self) -> &dyn Any { self }

    /// 1. 返回队列统计
    async fn queue_stats(&self) -> Vec<MatchStats> {
        let pending  = self.pending_tasks.lock().await;
        let waiting  = self.waiting_workers.lock().await;
        pending.iter().map(|(queue, tasks)| MatchStats {
            queue: queue.clone(),
            pending_tasks: tasks.len(),
            waiting_workers: waiting.len(),
        }).collect()
    }

    /// 2. poll：先取 pending，否则挂起等待
    async fn poll_task(
        &self,
        queue: &str,
        worker_id: &str,
        wait_time: Duration,
    ) -> Option<QueueTaskDto> {
        // 2.1 看内存 pending
        if let Some(task) = self.pending_tasks
            .lock().await
            .get_mut(queue)
            .and_then(|q| q.pop_front())
        {
            return Some(task);
        }

        // 2.2 挂起等待
        let (tx, rx) = oneshot::channel();
        self.waiting_workers
            .lock().await
            .insert(worker_id.to_string(), tx);
        match timeout(wait_time, rx).await {
            Ok(Ok(task)) => Some(task),
            _ => {
                self.waiting_workers.lock().await.remove(worker_id);
                None
            }
        }
    }

    /// 3. enqueue：优先派发给等待的 worker，否则放 pending
    ///
    /// 返回值是入队后的 `task_id`
    async fn enqueue_task(
        &self,
        queue: &str,
        task: QueueTaskDto,
    ) -> Result<String, String> {
        let task_id = task.task_id.clone();
        // 如果有等候的 worker，就直接发送
        if let Some((_, waiter)) = self.waiting_workers.lock().await.drain().next() {
            let _ = waiter.send(task);
            return Ok(task_id);
        }
        // 否则进 pending
        self.pending_tasks
            .lock().await
            .entry(queue.to_string())
            .or_default()
            .push_back(task);
        Ok(task_id)
    }

    /// 4. mark_task_processing：内存层无需额外操作
    async fn mark_task_processing(
        &self,
        _queue: &str,
        _task_id: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    /// 5. wait_for_completion：内存版直接回显 input
    async fn wait_for_completion(
        &self,
        _run_id: &str,
        _state_name: &str,
        input: &Value,
        _pm: &DynPM,
    ) -> Result<Value, String> {
        Ok(input.clone())
    }
}
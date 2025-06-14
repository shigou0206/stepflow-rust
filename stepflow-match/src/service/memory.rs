// src/service/memory_match_service.rs
//! 纯内存任务匹配服务 —— 适配最新版 MatchService Trait

use std::{
    any::Any,
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use tokio::{
    sync::{oneshot, Mutex},
    time::timeout,
};

use crate::service::interface::{DynPM, MatchService};
use stepflow_dto::dto::{
    match_stats::MatchStats,
    queue_task::{QueueTaskDto, UpdateQueueTaskDto},
};

/// 线程安全的内存实现，适合单机测试/开发
pub struct MemoryMatchService {
    /// queue → pending 任务
    pending_tasks: Mutex<HashMap<String, VecDeque<QueueTaskDto>>>,
    /// worker_id → 挂起中的 oneshot Sender
    waiting_workers: Mutex<HashMap<String, oneshot::Sender<QueueTaskDto>>>,
    /// (run_id, state_name) → 完成输出
    finished_results: Mutex<HashMap<(String, String), Value>>,
}

impl MemoryMatchService {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            pending_tasks:    Mutex::new(HashMap::new()),
            waiting_workers:  Mutex::new(HashMap::new()),
            finished_results: Mutex::new(HashMap::new()),
        })
    }
}

#[async_trait]
impl MatchService for MemoryMatchService {
    // ───────── utils ─────────
    fn as_any(&self) -> &dyn Any { self }

    async fn queue_stats(&self) -> Vec<MatchStats> {
        let pending = self.pending_tasks.lock().await;
        let waiting = self.waiting_workers.lock().await;

        pending
            .iter()
            .map(|(q, list)| MatchStats {
                queue: q.clone(),
                pending_tasks: list.len(),
                waiting_workers: waiting.len(),
            })
            .collect()
    }

    // ───────── enqueue ───────
    async fn enqueue_task(
        &self,
        queue: &str,
        mut task: QueueTaskDto,
    ) -> Result<String, String> {
        let task_id = task.task_id.clone();

        // 有 worker 等待就立即派发
        if let Some((_, waiter)) = self.waiting_workers.lock().await.drain().next() {
            task.status        = "processing".into();
            task.processing_at = Some(Utc::now());
            let _ = waiter.send(task);
            return Ok(task_id);
        }

        // 否则放进 pending
        self.pending_tasks
            .lock()
            .await
            .entry(queue.to_string())
            .or_default()
            .push_back(task);
        Ok(task_id)
    }

    // ───────── take_task ─────
    async fn take_task(
        &self,
        queue: &str,
        worker_id: &str,
        wait: Duration,
    ) -> Option<QueueTaskDto> {
        // ① 现有 pending
        if let Some(mut task) = self
            .pending_tasks
            .lock()
            .await
            .get_mut(queue)
            .and_then(|q| q.pop_front())
        {
            task.status        = "processing".into();
            task.processing_at = Some(Utc::now());
            return Some(task);
        }

        // ② 没任务 → 长轮询
        let (tx, rx) = oneshot::channel();
        self.waiting_workers
            .lock()
            .await
            .insert(worker_id.to_string(), tx);

        match timeout(wait, rx).await {
            Ok(Ok(task)) => Some(task),
            _ => {
                self.waiting_workers.lock().await.remove(worker_id);
                None
            }
        }
    }

    // ───────── finish_task ───
    async fn finish_task(
        &self,
        run_id:     &str,
        state_name: &str,
        patch:      UpdateQueueTaskDto,
    ) -> Result<(), String> {
        // 只有 status = completed 且带 task_payload 时才缓存结果
        if matches!(patch.status.as_deref(), Some("completed")) {
            if let Some(Some(output)) = patch.task_payload {
                self.finished_results
                    .lock()
                    .await
                    .insert((run_id.to_string(), state_name.to_string()), output);
            }
        }
        Ok(())
    }

    // ───────── wait_for_completion ─
    async fn wait_for_completion(
        &self,
        run_id: &str,
        state_name: &str,
        input_snapshot: &Value,
        _pm: &DynPM,
    ) -> Result<Value, String> {
        if let Some(payload) = self
            .finished_results
            .lock()
            .await
            .remove(&(run_id.to_string(), state_name.to_string()))
        {
            return Ok(payload);
        }
        // 对纯内存实现：未完成则返回进入节点时的快照
        Ok(input_snapshot.clone())
    }
}
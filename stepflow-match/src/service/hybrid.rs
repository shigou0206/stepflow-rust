//! service/hybrid_match_service.rs

use std::{any::Any, sync::Arc, time::Duration};
use async_trait::async_trait;
use serde_json::Value;
use tracing::debug;

use crate::service::interface::{DynPM, MatchService};
use stepflow_dto::dto::queue_task::QueueTaskDto;

/// 把内存队列与持久化队列“混合”在一起的 Service
pub struct HybridMatchService {
    memory_service:     Arc<dyn MatchService>,
    persistent_service: Arc<dyn MatchService>,
    fallback_enabled:   bool,
}

impl HybridMatchService {
    pub fn new(
        memory_service:     Arc<dyn MatchService>,
        persistent_service: Arc<dyn MatchService>,
    ) -> Arc<Self> {
        Arc::new(Self {
            memory_service,
            persistent_service,
            fallback_enabled: false,
        })
    }
}

#[async_trait]
impl MatchService for HybridMatchService {
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// poll 的同时立即将任务标记为 processing
    async fn poll_task(
        &self,
        queue: &str,
        worker_id: &str,
        timeout: Duration,
    ) -> Option<QueueTaskDto> {
        // 1️⃣ 先尝试内存队列
        if let Some(task) = self
            .memory_service
            .poll_task(queue, worker_id, timeout)
            .await
        {
            debug!("poll task: {:?}", task);
            // **同步** 标记持久化层
            debug!("mark task processing: {}", task.task_id);
            if let Err(e) = self
                .persistent_service
                .mark_task_processing(queue, &task.task_id)
                .await
            {
                tracing::warn!(
                    "Failed to mark processing for {} in persistent: {}",
                    task.task_id, e
                );
            }
            return Some(task);
        }

        // 2️⃣ 回退到持久化队列
        if self.fallback_enabled {
            self.persistent_service.poll_task(queue, worker_id, timeout).await
        } else {
            None
        }
    }

    /// enqueue 双写：先持久化获得 task_id，再写内存队列
    async fn enqueue_task(
        &self,
        queue: &str,
        mut task: QueueTaskDto,
    ) -> Result<String, String> {
        // 持久化入库，返回新的 task_id
        let new_id = self
            .persistent_service
            .enqueue_task(queue, task.clone())
            .await
            .map_err(|e| format!("persist enqueue failed: {e}"))?;
        // 回填 ID
        task.task_id = new_id.clone();
        // 写入内存队列
        self.memory_service
            .enqueue_task(queue, task)
            .await
            .map_err(|e| format!("memory enqueue failed: {e}"))?;
        Ok(new_id)
    }

    /// 标记持久化层任务为 processing
    async fn mark_task_processing(
        &self,
        queue: &str,
        task_id: &str,
    ) -> Result<(), String> {
        self.persistent_service.mark_task_processing(queue, task_id).await
    }

    /// 等待完成委托给内存层
    async fn wait_for_completion(
        &self,
        run_id: &str,
        state_name: &str,
        input: &Value,
        pm: &DynPM,
    ) -> Result<Value, String> {
        self.memory_service
            .wait_for_completion(run_id, state_name, input, pm)
            .await
    }
}
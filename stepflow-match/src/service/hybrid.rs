//! service/hybrid_match_service.rs
//!  - “内存优先 + 持久化兜底” 的调度层
//!  - 适配最新版 MatchService Trait

use std::{any::Any, sync::Arc, time::Duration};

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use tracing::{debug, warn};

use crate::service::interface::{DynPM, MatchService};
use stepflow_dto::dto::{
    queue_task::{QueueTaskDto, UpdateQueueTaskDto},
};

/// 内存 + 持久化混合实现
pub struct HybridMatchService {
    memory_service:     Arc<dyn MatchService>,
    persistent_service: Arc<dyn MatchService>,
    fallback_enabled:   bool,  // 内存没命中时，是否回落到持久化
}

impl HybridMatchService {
    pub fn new(
        memory_service:     Arc<dyn MatchService>,
        persistent_service: Arc<dyn MatchService>,
    ) -> Arc<Self> {
        Arc::new(Self {
            memory_service,
            persistent_service,
            fallback_enabled: true,
        })
    }
}

#[async_trait]
impl MatchService for HybridMatchService {
    // 允许向下转型
    fn as_any(&self) -> &dyn Any { self }

    // ────────── enqueue ─────────────────────────────────────────
    /// 1️⃣ 先写持久化获取 task_id → 2️⃣ 回写内存（回填同一个 task_id）
    async fn enqueue_task(
        &self,
        queue: &str,
        mut task: QueueTaskDto,
    ) -> Result<String, String> {
        // ① 落库
        let task_id = self
            .persistent_service
            .enqueue_task(queue, task.clone())
            .await?;

        // ② 把同一 task 放入内存
        task.task_id = task_id.clone();
        self.memory_service
            .enqueue_task(queue, task)
            .await
            .map_err(|e| format!("enqueue to memory failed: {e}"))?;

        Ok(task_id)
    }

    // ────────── take_task ───────────────────────────────────────
    /// 先尝试「内存」，若 miss 且允许则退到持久化。
    async fn take_task(
        &self,
        queue: &str,
        worker_id: &str,
        timeout: Duration,
    ) -> Option<QueueTaskDto> {
        // ① 内存队列
        if let Some(task) = self
            .memory_service
            .take_task(queue, worker_id, timeout)
            .await
        {
            debug!("take (memory) -> {:?}", task);

            // 同步把持久化那条改成 processing
            let patch = UpdateQueueTaskDto {
                status:        Some("processing".into()),
                processing_at: Some(Some(Utc::now())),
                ..Default::default()
            };
            if let Err(e) = self
                .persistent_service
                .finish_task(&task.run_id, &task.state_name, patch)
                .await
            {
                warn!("shadow mark processing failed: {e}");
            }

            return Some(task);
        }

        // ② 持久化队列（兜底）
        if self.fallback_enabled {
            if let Some(task) = self
                .persistent_service
                .take_task(queue, worker_id, timeout)
                .await
            {
                debug!("take (persistent) -> {:?}", task);

                // 放一份到内存，方便 wait/notify
                let _ = self.memory_service.enqueue_task(queue, task.clone()).await;
                return Some(task);
            }
        }

        None
    }

    // ────────── finish_task ─────────────────────────────────────
    /// 先持久化写成功，再补写内存；内存失败只告警不阻塞。
    async fn finish_task(
        &self,
        run_id:     &str,
        state_name: &str,
        patch:      UpdateQueueTaskDto,
    ) -> Result<(), String> {
        // ① 持久化必须成功
        self.persistent_service
            .finish_task(run_id, state_name, patch.clone())
            .await?;

        // ② 内存层若失败，仅记录 warn
        if let Err(e) = self
            .memory_service
            .finish_task(run_id, state_name, patch)
            .await
        {
            warn!("memory finish_task failed: {e}");
        }

        Ok(())
    }

    // ────────── wait_for_completion ────────────────────────────
    /// 仍由内存实现最快返回
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
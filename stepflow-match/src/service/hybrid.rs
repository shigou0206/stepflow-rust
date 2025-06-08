//! service/hybrid.rs

use std::{any::Any, sync::Arc, time::Duration};

use async_trait::async_trait;
use serde_json::Value;

use crate::service::interface::{DynPM, MatchService};
use stepflow_dto::dto::queue_task::QueueTaskDto;

/// 把内存队列与持久化队列“混合”在一起的 Service
pub struct HybridMatchService {
    memory_service:     Arc<dyn MatchService>,
    persistent_service: Arc<dyn MatchService>,
    /// 若内存队列取不到任务时，是否回退到持久化队列
    fallback_enabled: bool,
}

impl HybridMatchService {
    pub fn new(
        memory_service: Arc<dyn MatchService>,
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
    fn as_any(&self) -> &dyn Any {
        self
    }

    // ───────────────── poll / enqueue ─────────────────

    async fn poll_task(
        &self,
        queue: &str,
        worker_id: &str,
        timeout: Duration,
    ) -> Option<QueueTaskDto> {
        // 1️⃣ 先尝试内存队列
        if let Some(t) = self
            .memory_service
            .poll_task(queue, worker_id, timeout)
            .await
        {
            return Some(t);
        }
        // 2️⃣ 回退到持久化队列
        if self.fallback_enabled {
            self.persistent_service
                .poll_task(queue, worker_id, timeout)
                .await
        } else {
            None
        }
    }

    async fn enqueue_task(&self, queue: &str, task: QueueTaskDto) -> Result<(), String> {
        // 两边都放一份（确保持久化）
        self.memory_service.enqueue_task(queue, task.clone()).await?;
        self.persistent_service.enqueue_task(queue, task).await
    }

    // ──────────────── wait_for_completion ────────────────
    async fn wait_for_completion(
        &self,
        run_id: &str,
        state_name: &str,
        input: &Value,
        pm: &DynPM,                       // ← 新接口：&DynPM
    ) -> Result<Value, String> {
        // 直接委托给内存实现（如有需要也可加持久化逻辑）
        self.memory_service
            .wait_for_completion(run_id, state_name, input, pm)
            .await
    }
}
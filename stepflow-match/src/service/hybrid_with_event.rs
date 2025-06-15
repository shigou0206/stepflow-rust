use std::{any::Any, sync::Arc, time::Duration};
use async_trait::async_trait;
use serde_json::Value;

use crate::service::interface::{DynPM, MatchService};
use stepflow_dto::dto::{
    queue_task::{QueueTaskDto, UpdateQueueTaskDto},
    engine_event::EngineEvent,
};
use stepflow_eventbus::core::bus::EventBus;

/// 事件驱动 + 持久化混合实现
pub struct HybridMatchServiceWithEvent {
    persistent_service: Arc<dyn MatchService>,
    event_bus: Arc<dyn EventBus>,
}

impl HybridMatchServiceWithEvent {
    pub fn new(
        persistent_service: Arc<dyn MatchService>,
        event_bus: Arc<dyn EventBus>,
    ) -> Arc<Self> {
        Arc::new(Self {
            persistent_service,
            event_bus,
        })
    }
}

#[async_trait]
impl MatchService for HybridMatchServiceWithEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn enqueue_task(
        &self,
        queue: &str,
        task: QueueTaskDto,
    ) -> Result<String, String> {
        let task_id = self
            .persistent_service
            .enqueue_task(queue, task.clone())
            .await?;

        // —— 发事件给 EventBus（Worker 订阅处理） ——
        let event = EngineEvent::TaskReady {
            run_id: task.run_id.clone(),
            state_name: task.state_name.clone(),
            resource: task.resource.clone(),
            input: task.task_payload.clone(),
        };
        let _ = self.event_bus.emit(event.into());

        Ok(task_id)
    }

    async fn take_task(
        &self,
        _queue: &str,
        _worker_id: &str,
        _timeout: Duration,
    ) -> Option<QueueTaskDto> {
        // ⚠️ 在事件驱动模式中，不再使用轮询获取任务
        None
    }

    async fn finish_task(
        &self,
        run_id: &str,
        state_name: &str,
        patch: UpdateQueueTaskDto,
    ) -> Result<(), String> {
        self.persistent_service
            .finish_task(run_id, state_name, patch.clone())
            .await
    }

    async fn wait_for_completion(
        &self,
        run_id: &str,
        state_name: &str,
        input: &Value,
        pm: &DynPM,
    ) -> Result<Value, String> {
        self.persistent_service
            .wait_for_completion(run_id, state_name, input, pm)
            .await
    }
}
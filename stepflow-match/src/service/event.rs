use std::{any::Any, sync::Arc, time::Duration};
use async_trait::async_trait;
use serde_json::Value;
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_dto::dto::queue_task::{QueueTaskDto, UpdateQueueTaskDto};
use stepflow_dto::dto::match_stats::MatchStats;
use stepflow_eventbus::core::bus::EventBus;
use crate::service::interface::{DynPM, MatchService};

#[derive(Debug)]
pub struct EventDrivenMatchService {
    event_bus: Arc<dyn EventBus>,
}

impl EventDrivenMatchService {
    pub fn new(event_bus: Arc<dyn EventBus>) -> Arc<Self> {
        Arc::new(Self { event_bus })
    }
}

#[async_trait]
impl MatchService for EventDrivenMatchService {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn queue_stats(&self) -> Vec<MatchStats> {
        vec![] // optional
    }

    async fn enqueue_task(&self, _queue: &str, task: QueueTaskDto) -> Result<String, String> {
        let task_id = task.task_id.clone();

        let event = EngineEvent::TaskReady {
            run_id: task.run_id.clone(),
            state_name: task.state_name.clone(),
            resource: task.resource.clone(),
            input: task.task_payload.clone(),
        };

        self.event_bus
            .emit(event.into())
            .map_err(|e| format!("Event emit failed: {e}"))?;

        Ok(task_id)
    }

    async fn take_task(
        &self,
        _queue: &str,
        _worker_id: &str,
        _timeout: Duration,
    ) -> Option<QueueTaskDto> {
        None
    }

    async fn finish_task(
        &self,
        _run_id: &str,
        _state_name: &str,
        _patch: UpdateQueueTaskDto,
    ) -> Result<(), String> {
        Ok(())
    }

    async fn wait_for_completion(
        &self,
        _run_id: &str,
        _state_name: &str,
        input_snapshot: &Value,
        _pm: &DynPM,
    ) -> Result<Value, String> {
        Ok(input_snapshot.clone())
    }
}
use std::sync::Arc;
use async_trait::async_trait;
use stepflow_eventbus::core::bus::EventBus;
use stepflow_dto::dto::engine_event::EngineEvent;
use crate::service::interface::SubflowMatchService;
use serde_json::Value;

pub struct EventDrivenSubflowMatchService {
    event_bus: Arc<dyn EventBus>,
}

impl EventDrivenSubflowMatchService {
    pub fn new(event_bus: Arc<dyn EventBus>) -> Arc<Self> {
        Arc::new(Self { event_bus })
    }
}

#[async_trait]
impl SubflowMatchService for EventDrivenSubflowMatchService {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn notify_subflow_ready(
        &self,
        run_id: String,
        parent_run_id: String,
        state_name: String,
        dsl: Value,
        init_ctx: Value,
    ) -> Result<(), String> {
        let event = EngineEvent::SubflowReady {
            run_id,
            parent_run_id,
            state_name,
            dsl,
            init_ctx,
        };
        self.event_bus
            .emit(event.into())
            .map_err(|e| format!("emit failed: {e}"))
    }
}
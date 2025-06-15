use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;
use crate::error::EventBusError;
use stepflow_dto::dto::event_envelope::EventEnvelope;
use stepflow_dto::dto::engine_event::EngineEvent;
use std::fmt::Debug;

#[async_trait]
pub trait EventBus: Send + Sync + Debug {
    fn emit(&self, event: EventEnvelope) -> Result<(), EventBusError>;
    fn subscribe(&self) -> Receiver<EventEnvelope>;

    /// 高阶语义事件：EngineEvent（工作流引擎核心）
    async fn publish_engine_event(&self, event: EngineEvent) -> Result<(), EventBusError>;
}


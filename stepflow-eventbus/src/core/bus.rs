use async_trait::async_trait;
use tokio::sync::broadcast::Receiver;
use crate::error::EventBusError;
use stepflow_dto::dto::event_envelope::EventEnvelope;
use std::fmt::Debug;

#[async_trait]
pub trait EventBus: Send + Sync + Debug {
    fn emit(&self, event: EventEnvelope) -> Result<(), EventBusError>;
    fn subscribe(&self) -> Receiver<EventEnvelope>;
}
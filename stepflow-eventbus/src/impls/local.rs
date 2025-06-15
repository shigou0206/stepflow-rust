use tokio::sync::broadcast::{channel, Sender, Receiver};
use crate::core::bus::EventBus;
use crate::error::EventBusError;
use stepflow_dto::dto::event_envelope::EventEnvelope;
use stepflow_dto::dto::engine_event::EngineEvent;

#[derive(Clone, Debug)]
pub struct LocalEventBus {
    sender: Sender<EventEnvelope>,
}

impl LocalEventBus {
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = channel(buffer);
        Self { sender }
    }
}

#[async_trait::async_trait]
impl EventBus for LocalEventBus {
    fn emit(&self, event: EventEnvelope) -> Result<(), EventBusError> {
        self.sender
            .send(event)
            .map(|_| ())
            .map_err(|e| EventBusError::BroadcastError(e.to_string()))
    }

    fn subscribe(&self) -> Receiver<EventEnvelope> {
        self.sender.subscribe()
    }

    async fn publish_engine_event(&self, event: EngineEvent) -> Result<(), EventBusError> {
        let envelope: EventEnvelope = event.into();  // 直接用 From
        self.emit(envelope)
    }
}
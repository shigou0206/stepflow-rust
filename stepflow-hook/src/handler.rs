use crate::event::EngineEvent;

#[async_trait::async_trait]
pub trait EngineEventHandler: Send + Sync {
    async fn handle_event(&self, event: EngineEvent);
}
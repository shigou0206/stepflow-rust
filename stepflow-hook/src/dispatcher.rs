use crate::{EngineEvent, EngineEventHandler};
use std::sync::Arc;

pub struct EngineEventDispatcher {
    handlers: Vec<Arc<dyn EngineEventHandler>>,
}

impl EngineEventDispatcher {
    pub fn new(handlers: Vec<Arc<dyn EngineEventHandler>>) -> Self {
        Self { handlers }
    }

    pub async fn dispatch(&self, event: EngineEvent) {
        for handler in &self.handlers {
            handler.handle_event(event.clone()).await;
        }
    }
}
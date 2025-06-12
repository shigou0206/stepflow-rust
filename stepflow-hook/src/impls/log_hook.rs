use crate::{EngineEventHandler};
use std::sync::Arc;
use stepflow_dto::dto::engine_event::EngineEvent;

pub struct LogHook;

impl LogHook {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait::async_trait]
impl EngineEventHandler for LogHook {
    async fn handle_event(&self, event: EngineEvent) {
        println!("[LogHook] {:?}", event);
    }
}
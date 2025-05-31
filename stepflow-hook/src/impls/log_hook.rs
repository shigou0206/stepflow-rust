use crate::{EngineEvent, EngineEventHandler};
use std::sync::Arc;

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
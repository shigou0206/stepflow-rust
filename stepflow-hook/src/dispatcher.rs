use crate::{EngineEvent, EngineEventHandler};
use std::sync::Arc;
use tokio::sync::mpsc;
use std::time::Duration;

pub struct EngineEventDispatcher {
    handlers: Vec<Arc<dyn EngineEventHandler>>,
    batch_sender: Option<mpsc::UnboundedSender<EngineEvent>>,
}

impl EngineEventDispatcher {
    pub fn new(handlers: Vec<Arc<dyn EngineEventHandler>>) -> Self {
        Self { 
            handlers,
            batch_sender: None,
        }
    }

    pub fn enable_batch_processing(mut self, batch_size: usize, flush_interval: Duration) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let handlers = self.handlers.clone();
        
        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(batch_size);
            let mut interval = tokio::time::interval(flush_interval);
            
            loop {
                tokio::select! {
                    Some(event) = rx.recv() => {
                        batch.push(event);
                        if batch.len() >= batch_size {
                            Self::process_batch(&handlers, batch).await;
                            batch = Vec::with_capacity(batch_size);
                        }
                    }
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            Self::process_batch(&handlers, batch).await;
                            batch = Vec::with_capacity(batch_size);
                        }
                    }
                }
            }
        });

        self.batch_sender = Some(tx);
        self
    }

    async fn process_batch(handlers: &[Arc<dyn EngineEventHandler>], batch: Vec<EngineEvent>) {
        for handler in handlers {
            for event in &batch {
                handler.handle_event(event.clone()).await;
            }
        }
    }

    pub async fn dispatch(&self, event: EngineEvent) {
        if let Some(tx) = &self.batch_sender {
            let _ = tx.send(event);
        } else {
            for handler in &self.handlers {
                handler.handle_event(event.clone()).await;
            }
        }
    }
}
use crate::EngineEventHandler;
use std::sync::Arc;
use std::time::Duration;
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_eventbus::core::bus::EventBus;
use tokio::sync::mpsc;

pub struct EngineEventDispatcher {
    handlers: Vec<Arc<dyn EngineEventHandler>>,
    batch_sender: Option<mpsc::UnboundedSender<EngineEvent>>,
    bus: Arc<dyn EventBus>,
}

impl EngineEventDispatcher {
    pub fn new(handlers: Vec<Arc<dyn EngineEventHandler>>, bus: Arc<dyn EventBus>) -> Self {
        Self {
            handlers,
            batch_sender: None,
            bus,
        }
    }

    pub fn enable_batch_processing(mut self, batch_size: usize, flush_interval: Duration) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let handlers = self.handlers.clone();
        let bus = self.bus.clone();

        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(batch_size);
            let mut interval = tokio::time::interval(flush_interval);

            loop {
                tokio::select! {
                    Some(event) = rx.recv() => {
                        batch.push(event);
                        if batch.len() >= batch_size {
                            Self::process_batch(&handlers, &bus, batch).await;
                            batch = Vec::with_capacity(batch_size);
                        }
                    }
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            Self::process_batch(&handlers, &bus, batch).await;
                            batch = Vec::with_capacity(batch_size);
                        }
                    }
                }
            }
        });

        self.batch_sender = Some(tx);
        self
    }

    async fn process_batch(
        handlers: &[Arc<dyn EngineEventHandler>],
        bus: &Arc<dyn EventBus>,
        batch: Vec<EngineEvent>,
    ) {
        for event in &batch {
            for handler in handlers {
                handler.handle_event(event.clone()).await;
            }

            let _ = bus.emit(event.clone().into());
        }
    }

    pub async fn dispatch(&self, event: EngineEvent) {
        if let Some(tx) = &self.batch_sender {
            let _ = tx.send(event.clone());
        } else {
            for handler in &self.handlers {
                handler.handle_event(event.clone()).await;
            }

            let _ = self.bus.emit(event.clone().into());
        }
    }
}
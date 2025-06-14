use crate::EngineEventHandler;
use std::sync::Arc;
use std::time::Duration;
use sqlx::types::uuid;
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_dto::dto::event_envelope::EventEnvelope;
use stepflow_eventbus::core::bus::EventBus;
use tokio::sync::mpsc;
use chrono::Utc;
use uuid::Uuid;

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
            let _ = tx.send(event.clone());
        } else {
            for handler in &self.handlers {
                handler.handle_event(event.clone()).await;
            }
        }

        // —— 再广播给 EventBus ——
        let run_id = match &event {
            EngineEvent::WorkflowStarted { run_id }
            | EngineEvent::WorkflowFinished { run_id, .. }
            | EngineEvent::NodeEnter { run_id, .. }
            | EngineEvent::NodeSuccess { run_id, .. }
            | EngineEvent::NodeFailed { run_id, .. }
            | EngineEvent::NodeCancelled { run_id, .. }
            | EngineEvent::NodeExit { run_id, .. }
            | EngineEvent::NodeDispatched { run_id, .. }
            | EngineEvent::TimerScheduled { run_id, .. }
            | EngineEvent::TimerFired { run_id, .. }
            | EngineEvent::ActivityTaskDispatched { run_id, .. }
            | EngineEvent::ActivityTaskCompleted { run_id, .. }
            | EngineEvent::UiEventPushed { run_id, .. } => run_id.clone(),
        };

        let envelope = EventEnvelope {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            run_id,
            event,
        };
        let _ = self.bus.emit(envelope);
    }
}

use std::{sync::Arc, time::Duration};
use async_trait::async_trait;
use serde_json::Value;
use std::any::Any;
use stepflow_storage::persistence_manager::PersistenceManager;
use crate::service::interface::{MatchService, Task};

/// HybridMatchService: combines memory-based and persistent task queues
pub struct HybridMatchService {
    memory_service: Arc<dyn MatchService>,
    persistent_service: Arc<dyn MatchService>,
    fallback_enabled: bool,
}

impl HybridMatchService {
    pub fn new(
        memory_service: Arc<dyn MatchService>,
        persistent_service: Arc<dyn MatchService>,
    ) -> Arc<Self> {
        Arc::new(Self {
            memory_service,
            persistent_service,
            fallback_enabled: true,
        })
    }
}

#[async_trait]
impl MatchService for HybridMatchService {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn poll_task(&self, queue: &str, worker_id: &str, timeout: Duration) -> Option<Task> {
        // Step 1: Try memory
        if let Some(task) = self.memory_service.poll_task(queue, worker_id, timeout).await {
            return Some(task);
        }

        // Step 2: Try persistent queue if fallback is enabled
        if self.fallback_enabled {
            self.persistent_service.poll_task(queue, worker_id, timeout).await
        } else {
            None
        }
    }

    async fn enqueue_task(&self, queue: &str, task: Task) -> Result<(), String> {
        // Try both memory and persistent
        self.memory_service.enqueue_task(queue, task.clone()).await?;
        self.persistent_service.enqueue_task(queue, task).await
    }

    async fn wait_for_completion(
        &self,
        run_id: &str,
        state_name: &str,
        input: &Value,
        persistence: Arc<dyn PersistenceManager>,
    ) -> Result<Value, String> {
        self.memory_service.wait_for_completion(run_id, state_name, input, persistence).await
    }
}
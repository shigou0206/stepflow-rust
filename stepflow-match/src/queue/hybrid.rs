use std::sync::Arc;
use async_trait::async_trait;
use stepflow_storage::persistence_manager::PersistenceManager;
use crate::queue::memory::MemoryQueue;
use crate::queue::persistent::PersistentStore;
use crate::queue::traits::TaskStore;
use crate::queue::traits::TaskQueue;

/// 混合队列：写入 memory + persistent store，读取优先 memory
pub struct HybridQueue {
    memory: Arc<MemoryQueue>,
    store: Arc<PersistentStore>,
}

impl HybridQueue {
    pub fn new(
        memory: Arc<MemoryQueue>,
        store: Arc<PersistentStore>,
    ) -> Self {
        Self { memory, store }
    }
}

#[async_trait]
impl TaskQueue for HybridQueue {
    async fn push(
        &self,
        persistence: &Arc<dyn PersistenceManager>,
        run_id: &str,
        state_name: &str,
    ) -> Result<(), String> {
        self.memory.push(persistence, run_id, state_name).await?;
        self.store.insert_task(persistence, run_id, state_name, "", &serde_json::json!({})).await
    }

    async fn pop(
        &self,
        persistence: &Arc<dyn PersistenceManager>,
    ) -> Result<Option<(String, String)>, String> {
        self.memory.pop(persistence).await
    }
}
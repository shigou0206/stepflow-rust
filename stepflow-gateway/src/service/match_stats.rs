use std::collections::HashMap;
use std::sync::Arc;
use crate::dto::match_stats::{MatchStatsResponse, MemoryStats, DbStats, QueueTaskPreview};
use stepflow_match::queue::MemoryQueue;
use stepflow_match::queue::TaskQueue;
use stepflow_storage::persistence_manager::PersistenceManager;

#[derive(Clone)]
pub struct MatchService {
    pub queue: Arc<dyn TaskQueue>,
    pub memory: Arc<MemoryQueue>,
    pub persistence: Arc<dyn PersistenceManager>,
}

impl MatchService {
    pub fn new(
        queue: Arc<dyn TaskQueue>,
        memory: Arc<MemoryQueue>,
        persistence: Arc<dyn PersistenceManager>,
    ) -> Self {
        Self { queue, memory, persistence }
    }

    pub async fn memory_stats(&self) -> Result<MemoryStats, String> {
        let total = self.memory.len().await;
        let peek = self.memory.peek().await;

        Ok(MemoryStats {
            total,
            peek: peek.map(|task| QueueTaskPreview {
                run_id: task.run_id,
                state_name: task.state_name,
                priority: task.priority,
                created_at: task.created_at,
            }),
        })
    }

    pub async fn db_stats(&self) -> Result<DbStats, String> {
        let mut count_map = HashMap::new();
        for status in ["pending", "processing", "completed", "failed", "retrying"] {
            let count = self.persistence
                .find_queue_tasks_by_status(status, 100_000, 0)
                .await
                .map(|tasks| tasks.len())
                .unwrap_or(0);
            count_map.insert(status.to_string(), count);
        }

        Ok(DbStats {
            pending: *count_map.get("pending").unwrap_or(&0),
            processing: *count_map.get("processing").unwrap_or(&0),
            completed: *count_map.get("completed").unwrap_or(&0),
            failed: *count_map.get("failed").unwrap_or(&0),
            retrying: *count_map.get("retrying").unwrap_or(&0),
        })
    }

    pub async fn collect_stats(&self) -> Result<MatchStatsResponse, String> {
        Ok(MatchStatsResponse {
            memory: self.memory_stats().await?,
            db: self.db_stats().await?,
        })
    }
}
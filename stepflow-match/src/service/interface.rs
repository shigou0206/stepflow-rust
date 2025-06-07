use async_trait::async_trait;
use std::time::Duration;
use serde_json::Value;
use std::sync::Arc;
use std::any::Any;
use stepflow_storage::persistence_manager::PersistenceManager;
use stepflow_dto::dto::match_stats::MatchStats;
use stepflow_dto::dto::queue_task::QueueTaskDto;


#[async_trait]
pub trait MatchService: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    async fn queue_stats(&self) -> Vec<MatchStats> {
        vec![]
    }

    async fn poll_task(&self, queue: &str, worker_id: &str, timeout: Duration) -> Option<QueueTaskDto>;
    async fn enqueue_task(&self, queue: &str, task: QueueTaskDto) -> Result<(), String>;
    async fn wait_for_completion(
        &self,
        run_id: &str,
        state_name: &str,
        input: &Value,
        persistence: Arc<dyn PersistenceManager>,
    ) -> Result<Value, String>;
}
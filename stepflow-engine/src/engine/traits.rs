use serde_json::Value;
use stepflow_storage::persistence_manager::PersistenceManager;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait TaskStore: Send + Sync {
    async fn insert_task(
        &self,
        persistence: &Arc<dyn PersistenceManager>,
        run_id: &str,
        state_name: &str,
        resource: &str,
        input: &Value,
    ) -> Result<(), String>;

    async fn update_task_status(
        &self,
        persistence: &Arc<dyn PersistenceManager>,
        run_id: &str,
        state_name: &str,
        status: &str,
        result: &Value,
    ) -> Result<(), String>;
}

#[async_trait::async_trait]
pub trait TaskQueue: Send + Sync {
    async fn push(
        &self,
        persistence: &Arc<dyn PersistenceManager>,
        run_id: &str,
        state_name: &str,
    ) -> Result<(), String>;

    async fn pop(
        &self,
        persistence: &Arc<dyn PersistenceManager>,
    ) -> Result<Option<(String, String)>, String>;
} 
//! TaskStore / TaskQueue trait —— 仅改动 persistence 参数的类型别名
use std::sync::Arc;
use serde_json::Value;

use stepflow_storage::{
    db::DbBackend,                        // 👈 统一后端
    persistence_manager::PersistenceManager,
};

pub type DynPM = Arc<dyn PersistenceManager<DB = DbBackend> + Send + Sync>;

#[async_trait::async_trait]
pub trait TaskStore: Send + Sync {
    async fn insert_task(
        &self,
        persistence: &DynPM,
        run_id: &str,
        state_name: &str,
        resource: &str,
        input: &Value,
    ) -> Result<(), String>;

    async fn update_task_status(
        &self,
        persistence: &DynPM,
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
        persistence: &DynPM,
        run_id: &str,
        state_name: &str,
    ) -> Result<(), String>;

    async fn pop(
        &self,
        persistence: &DynPM,
    ) -> Result<Option<(String, String)>, String>;
}
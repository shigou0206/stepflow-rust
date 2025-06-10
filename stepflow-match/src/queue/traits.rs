use serde_json::Value;

use stepflow_storage::db::DynPM;
use async_trait::async_trait;

#[async_trait]
pub trait TaskStore {
    /// 插入一条新任务，返回该行的 `task_id`
    async fn insert_task(
        &self,
        run_id: &str,
        state_name: &str,
        resource: &str,
        input: &Value,
    ) -> Result<String, String>;

    /// 更新任务状态（processing/completed/failed/...）
    async fn update_task_status(
        &self,
        run_id: &str,
        state_name: &str,
        new_status: &str,
        result: &Value,
    ) -> Result<(), String>;
}

#[async_trait]
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
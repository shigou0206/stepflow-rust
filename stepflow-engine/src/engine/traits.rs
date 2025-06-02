use serde_json::Value;
use sqlx::{Sqlite, Transaction};

#[async_trait::async_trait]
pub trait TaskStore: Send + Sync {
    async fn insert_task(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        run_id: &str,
        state_name: &str,
        resource: &str,
        input: &Value,
    ) -> Result<(), String>;

    async fn update_task_status(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
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
        tx: &mut Transaction<'_, Sqlite>,
        run_id: &str,
        state_name: &str,
    ) -> Result<(), String>;

    async fn pop(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> Result<Option<(String, String)>, String>;
} 
use sqlx::SqlitePool;
use stepflow_sqlite::{
    crud::activity_task_crud,
    models::activity_task::{ActivityTask, UpdateActivityTask},
};

#[derive(Clone)]
pub struct ActivityTaskPersistence {
    pool: SqlitePool,
}

impl ActivityTaskPersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_task(&self, task: &ActivityTask) -> Result<(), sqlx::Error> {
        activity_task_crud::create_task(&self.pool, task).await
    }

    pub async fn get_task(&self, task_token: &str) -> Result<Option<ActivityTask>, sqlx::Error> {
        activity_task_crud::get_task(&self.pool, task_token).await
    }

    pub async fn find_tasks_by_status(
        &self,
        status: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ActivityTask>, sqlx::Error> {
        activity_task_crud::find_tasks_by_status(&self.pool, status, limit, offset).await
    }

    pub async fn update_task(
        &self,
        task_token: &str,
        changes: &UpdateActivityTask,
    ) -> Result<(), sqlx::Error> {
        activity_task_crud::update_task(&self.pool, task_token, changes).await
    }

    pub async fn delete_task(&self, task_token: &str) -> Result<(), sqlx::Error> {
        activity_task_crud::delete_task(&self.pool, task_token).await
    }
}
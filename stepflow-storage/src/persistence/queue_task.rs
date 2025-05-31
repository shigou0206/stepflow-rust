use sqlx::SqlitePool;
use stepflow_sqlite::{
    crud::queue_task_crud,
    models::queue_task::{QueueTask, UpdateQueueTask},
};
use chrono::NaiveDateTime;

#[derive(Clone)]
pub struct QueueTaskPersistence {
    pool: SqlitePool,
}

impl QueueTaskPersistence {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_task(&self, task: &QueueTask) -> Result<(), sqlx::Error> {
        queue_task_crud::create_task(&self.pool, task).await
    }

    pub async fn get_task(&self, task_id: &str) -> Result<Option<QueueTask>, sqlx::Error> {
        queue_task_crud::get_task(&self.pool, task_id).await
    }

    pub async fn update_task(&self, task_id: &str, changes: &UpdateQueueTask) -> Result<(), sqlx::Error> {
        queue_task_crud::update_task(&self.pool, task_id, changes).await
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<(), sqlx::Error> {
        queue_task_crud::delete_task(&self.pool, task_id).await
    }

    pub async fn find_tasks_by_status(
        &self,
        status: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<QueueTask>, sqlx::Error> {
        queue_task_crud::find_tasks_by_status(&self.pool, status, limit, offset).await
    }

    // 新增根据重试时间(next_retry_at)查找待重试任务的方法
    pub async fn find_tasks_to_retry(
        &self,
        before: NaiveDateTime,
        limit: i64,
    ) -> Result<Vec<QueueTask>, sqlx::Error> {
        sqlx::query_as!(
            QueueTask,
            r#"
            SELECT task_id as "task_id!",
                   run_id as "run_id!",
                   state_name as "state_name!",
                   task_payload,
                   status as "status!",
                   attempts as "attempts!",
                   max_attempts as "max_attempts!",
                   error_message,
                   last_error_at,
                   next_retry_at,
                   queued_at as "queued_at!",
                   processing_at,
                   completed_at,
                   failed_at,
                   created_at as "created_at!",
                   updated_at as "updated_at!"
            FROM queue_tasks
            WHERE status = 'retrying' AND next_retry_at <= ?
            ORDER BY next_retry_at ASC
            LIMIT ?
            "#,
            before,
            limit
        )
        .fetch_all(&self.pool)
        .await
    }
}
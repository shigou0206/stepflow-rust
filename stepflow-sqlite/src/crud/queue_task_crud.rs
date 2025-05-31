use sqlx::{Executor, Sqlite, Result, QueryBuilder};
use crate::models::queue_task::{QueueTask, UpdateQueueTask};
use chrono::Utc;

// 创建任务
pub async fn create_task<'e, E>(executor: E, task: &QueueTask) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        r#"
        INSERT INTO queue_tasks (
            task_id, run_id, state_name, task_payload, status,
            attempts, max_attempts, error_message, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        task.task_id,
        task.run_id,
        task.state_name,
        task.task_payload,
        task.status,
        task.attempts,
        task.max_attempts,
        task.error_message,
        task.created_at,
        task.updated_at
    )
    .execute(executor).await?;
    Ok(())
}

// 获取指定 task_id 的任务（精确指定字段类型）
pub async fn get_task<'e, E>(executor: E, task_id: &str) -> Result<Option<QueueTask>>
where
    E: Executor<'e, Database = Sqlite>,
{
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
               created_at as "created_at!",
               updated_at as "updated_at!"
        FROM queue_tasks WHERE task_id = ?
        "#,
        task_id
    )
    .fetch_optional(executor).await
}

// 查找指定状态的任务（例如 pending 状态的任务）
pub async fn find_tasks_by_status<'e, E>(
    executor: E, status: &str, limit: i64, offset: i64
) -> Result<Vec<QueueTask>>
where
    E: Executor<'e, Database = Sqlite>,
{
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
               created_at as "created_at!",
               updated_at as "updated_at!"
        FROM queue_tasks
        WHERE status = ?
        ORDER BY created_at ASC
        LIMIT ? OFFSET ?
        "#,
        status,
        limit,
        offset
    )
    .fetch_all(executor).await
}

// 动态更新任务状态与相关字段（安全实现）
pub async fn update_task<'e, E>(
    executor: E, task_id: &str, changes: &UpdateQueueTask
) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let mut query = QueryBuilder::new("UPDATE queue_tasks SET ");
    let mut has_fields = false;

    macro_rules! set_field {
        ($field:ident) => {
            if let Some(val) = &changes.$field {
                if has_fields { query.push(", "); }
                query.push(stringify!($field)).push(" = ").push_bind(val);
                has_fields = true;
            }
        };
    }

    set_field!(status);
    set_field!(attempts);
    set_field!(error_message);

    // 确保至少更新 updated_at
    if has_fields {
        query.push(", ");
    }
    query.push("updated_at = ").push_bind(Utc::now().naive_utc());

    query.push(" WHERE task_id = ").push_bind(task_id);

    query.build().execute(executor).await?;
    Ok(())
}

// 删除任务记录
pub async fn delete_task<'e, E>(executor: E, task_id: &str) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        "DELETE FROM queue_tasks WHERE task_id = ?",
        task_id
    )
    .execute(executor).await?;
    Ok(())
}
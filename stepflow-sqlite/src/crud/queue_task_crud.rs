//! queue_task_crud.rs
//! CRUD helpers for `queue_tasks` table (SQLite)

use sqlx::{Executor, Result, Sqlite};
use crate::models::queue_task::{QueueTask, UpdateQueueTask};
use tracing::debug;
/// 1. create_task
pub async fn create_task<'e, E>(executor: E, task: &QueueTask) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        r#"
        INSERT INTO queue_tasks (
            task_id, run_id, state_name, resource, task_payload, status,
            attempts, max_attempts, priority, timeout_seconds,
            error_message, last_error_at, next_retry_at,
            queued_at, processing_at, completed_at, failed_at,
            created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        task.task_id,
        task.run_id,
        task.state_name,
        task.resource,
        task.task_payload,
        task.status,
        task.attempts,
        task.max_attempts,
        task.priority,
        task.timeout_seconds,
        task.error_message,
        task.last_error_at,
        task.next_retry_at,
        task.queued_at,
        task.processing_at,
        task.completed_at,
        task.failed_at,
        task.created_at,
        task.updated_at,
    )
    .execute(executor)
    .await?;
    Ok(())
}

/// 2. get_task
pub async fn get_task<'e, E>(executor: E, task_id: &str) -> Result<Option<QueueTask>>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        QueueTask,
        r#"
        SELECT
            task_id              AS "task_id!",
            run_id               AS "run_id!",
            state_name           AS "state_name!",
            resource             AS "resource!",
            task_payload,
            status               AS "status!",
            attempts             AS "attempts!",
            max_attempts         AS "max_attempts!",
            priority,
            timeout_seconds,
            error_message,
            last_error_at,
            next_retry_at,
            queued_at            AS "queued_at!",
            processing_at,
            completed_at,
            failed_at,
            created_at           AS "created_at!",
            updated_at           AS "updated_at!"
        FROM queue_tasks
        WHERE task_id = ?
        "#,
        task_id
    )
    .fetch_optional(executor)
    .await
}

/// 3. find_tasks_by_status
pub async fn find_tasks_by_status<'e, E>(
    executor: E,
    status: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<QueueTask>>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        QueueTask,
        r#"
        SELECT
            task_id              AS "task_id!",
            run_id               AS "run_id!",
            state_name           AS "state_name!",
            resource             AS "resource!",
            task_payload,
            status               AS "status!",
            attempts             AS "attempts!",
            max_attempts         AS "max_attempts!",
            priority,
            timeout_seconds,
            error_message,
            last_error_at,
            next_retry_at,
            queued_at            AS "queued_at!",
            processing_at,
            completed_at,
            failed_at,
            created_at           AS "created_at!",
            updated_at           AS "updated_at!"
        FROM queue_tasks
        WHERE status = ?
        ORDER BY queued_at ASC
        LIMIT ? OFFSET ?
        "#,
        status,
        limit,
        offset
    )
    .fetch_all(executor)
    .await
}

/// 4. update_task
///
/// 手动拼 SET 子句，边拼 SQL 边调用 `.bind(...)`，避免 trait-object 绑定
pub async fn update_task<'e, E>(
    executor: E,
    task_id: &str,
    changes: &UpdateQueueTask,
) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    // 先收集要更新的字段名
    let mut sets = Vec::new();

    if changes.status.is_some() {
        sets.push("status = ?");
    }
    if changes.attempts.is_some() {
        sets.push("attempts = ?");
    }
    if changes.priority.is_some() {
        sets.push("priority = ?");
    }
    if changes.timeout_seconds.is_some() {
        sets.push("timeout_seconds = ?");
    }

    // 双 Option 字段：None→NULL, Some(None)→SET NULL, Some(Some(v))→=v
    macro_rules! opt_opt {
        ($field:ident) => {
            if let Some(ref opt) = changes.$field {
                if opt.is_some() {
                    sets.push(concat!(stringify!($field), " = ?"));
                } else {
                    sets.push(concat!(stringify!($field), " = NULL"));
                }
            }
        };
    }
    opt_opt!(task_payload);
    opt_opt!(error_message);
    opt_opt!(last_error_at);
    opt_opt!(next_retry_at);
    opt_opt!(processing_at);
    opt_opt!(completed_at);
    opt_opt!(failed_at);

    // 如果一个更新字段都没有，就跳过
    if sets.is_empty() {
        return Ok(());
    }
    // 一定更新 updated_at
    sets.push("updated_at = CURRENT_TIMESTAMP");

    // 拼 SQL
    let sql = format!(
        "UPDATE queue_tasks SET {} WHERE task_id = ?",
        sets.join(", ")
    );
    debug!("update_task SQL = {}", sql);

    // 逐个 bind
    let mut q = sqlx::query(&sql);
    if let Some(v) = &changes.status {
        q = q.bind(v);
    }
    if let Some(v) = &changes.attempts {
        q = q.bind(v);
    }
    if let Some(v) = &changes.priority {
        q = q.bind(v);
    }
    if let Some(v) = &changes.timeout_seconds {
        q = q.bind(v);
    }
    macro_rules! bind_opt_opt {
        ($field:ident) => {
            if let Some(ref opt) = changes.$field {
                if let Some(val) = opt {
                    q = q.bind(val);
                }
            }
        };
    }
    bind_opt_opt!(task_payload);
    bind_opt_opt!(error_message);
    bind_opt_opt!(last_error_at);
    bind_opt_opt!(next_retry_at);
    bind_opt_opt!(processing_at);
    bind_opt_opt!(completed_at);
    bind_opt_opt!(failed_at);

    // 最后 bind 上 WHERE 的 task_id
    q = q.bind(task_id);

    // 执行
    q.execute(executor).await?;
    Ok(())
}

/// 5. delete_task
pub async fn delete_task<'e, E>(executor: E, task_id: &str) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("DELETE FROM queue_tasks WHERE task_id = ?", task_id)
        .execute(executor)
        .await?;
    Ok(())
}
//! queue_task_crud.rs
//! CRUD helpers for `queue_tasks` table (SQLite)

use sqlx::{Executor, QueryBuilder, Result, Sqlite};
use crate::models::queue_task::{QueueTask, UpdateQueueTask};

//// ------------------------------------------------------------------
//  1. create_task
//// ------------------------------------------------------------------

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
        VALUES ( ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ? )
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

//// ------------------------------------------------------------------
//  2. get_task
//// ------------------------------------------------------------------

pub async fn get_task<'e, E>(executor: E, task_id: &str) -> Result<Option<QueueTask>>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        QueueTask,
        r#"
        SELECT
            task_id              as "task_id!",
            run_id               as "run_id!",
            state_name           as "state_name!",
            resource             as "resource!",
            task_payload,
            status               as "status!",
            attempts             as "attempts!",
            max_attempts         as "max_attempts!",
            priority,
            timeout_seconds,
            error_message,
            last_error_at,
            next_retry_at,
            queued_at            as "queued_at!",
            processing_at,
            completed_at,
            failed_at,
            created_at           as "created_at!",
            updated_at           as "updated_at!"
        FROM queue_tasks
        WHERE task_id = ?
        "#,
        task_id
    )
    .fetch_optional(executor)
    .await
}

//// ------------------------------------------------------------------
//  3. find_tasks_by_status
//// ------------------------------------------------------------------

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
            task_id              as "task_id!",
            run_id               as "run_id!",
            state_name           as "state_name!",
            resource             as "resource!",
            task_payload,
            status               as "status!",
            attempts             as "attempts!",
            max_attempts         as "max_attempts!",
            priority,
            timeout_seconds,
            error_message,
            last_error_at,
            next_retry_at,
            queued_at            as "queued_at!",
            processing_at,
            completed_at,
            failed_at,
            created_at           as "created_at!",
            updated_at           as "updated_at!"
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

//// ------------------------------------------------------------------
//  4. update_task  (dynamic SQL; handles Option<Option<T>>)
//      - always sets `updated_at = CURRENT_TIMESTAMP`
//// ------------------------------------------------------------------

pub async fn update_task<'e, E>(
    executor: E,
    task_id: &str,
    changes: &UpdateQueueTask,
) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let mut qb  = QueryBuilder::new("UPDATE queue_tasks SET ");
    let mut sep = qb.separated(", ");
    let mut any = false;

    // one-level Option<T>
    macro_rules! set_opt {
        ($f:ident) => {
            if let Some(v) = &changes.$f {
                sep.push(format!("{} = ", stringify!($f))).push_bind(v);
                any = true;
            }
        };
    }
    // two-level Option<Option<T>>
    macro_rules! set_opt_opt {
        ($f:ident) => {
            if let Some(opt) = &changes.$f {
                match opt {
                    Some(v) => { sep.push(format!("{} = ", stringify!($f))).push_bind(v); }
                    None    => { sep.push(format!("{} = NULL", stringify!($f))); }
                }
                any = true;
            }
        };
    }

    set_opt!(status);
    set_opt!(attempts);
    set_opt_opt!(error_message);
    set_opt_opt!(last_error_at);
    set_opt_opt!(next_retry_at);
    set_opt_opt!(processing_at);
    set_opt_opt!(completed_at);
    set_opt_opt!(failed_at);

    // 总会更新 updated_at
    if any { sep.push("updated_at = CURRENT_TIMESTAMP"); } else { return Ok(()); }

    qb.push(" WHERE task_id = ").push_bind(task_id);
    qb.build().execute(executor).await?;
    Ok(())
}

//// ------------------------------------------------------------------
//  5. delete_task
//// ------------------------------------------------------------------

pub async fn delete_task<'e, E>(executor: E, task_id: &str) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("DELETE FROM queue_tasks WHERE task_id = ?", task_id)
        .execute(executor)
        .await?;
    Ok(())
}
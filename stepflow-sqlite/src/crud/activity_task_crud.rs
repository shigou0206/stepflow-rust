use sqlx::{Executor, QueryBuilder, Result, Sqlite};
use crate::models::activity_task::{ActivityTask, UpdateActivityTask};

// 创建新任务
pub async fn create_task<'e, E>(executor: E, task: &ActivityTask) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        r#"
        INSERT INTO activity_tasks (
            task_token, run_id, shard_id, seq, activity_type, state_name, input, result,
            status, error, error_details, attempt, max_attempts, heartbeat_at,
            scheduled_at, started_at, completed_at, timeout_seconds, retry_policy, version
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        task.task_token,
        task.run_id,
        task.shard_id,
        task.seq,
        task.activity_type,
        task.state_name,
        task.input,
        task.result,
        task.status,
        task.error,
        task.error_details,
        task.attempt,
        task.max_attempts,
        task.heartbeat_at,
        task.scheduled_at,
        task.started_at,
        task.completed_at,
        task.timeout_seconds,
        task.retry_policy,
        task.version
    )
    .execute(executor)
    .await?;
    Ok(())
}

// 查询单个任务（安全实现）
pub async fn get_task<'e, E>(executor: E, task_token: &str) -> Result<Option<ActivityTask>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let task = sqlx::query_as!(
        ActivityTask,
        r#"
        SELECT task_token as "task_token!", 
               run_id as "run_id!", 
               shard_id as "shard_id!", 
               seq as "seq!",
               activity_type as "activity_type!", 
               state_name, input, result,
               status as "status!", error, error_details, 
               attempt as "attempt!",
               max_attempts as "max_attempts!", 
               heartbeat_at, scheduled_at as "scheduled_at!",
               started_at, completed_at, 
               timeout_seconds, retry_policy, 
               version as "version!"
        FROM activity_tasks 
        WHERE task_token = ?
        "#,
        task_token
    )
    .fetch_optional(executor)
    .await?;

    Ok(task)
}

// 根据状态分页查询任务
pub async fn find_tasks_by_status<'e, E>(
    executor: E,
    status: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<ActivityTask>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let tasks = sqlx::query_as!(
        ActivityTask,
        r#"
        SELECT task_token as "task_token!", 
               run_id as "run_id!", shard_id as "shard_id!", seq as "seq!",
               activity_type as "activity_type!", state_name, input, result,
               status as "status!", error, error_details, attempt as "attempt!",
               max_attempts as "max_attempts!", heartbeat_at, scheduled_at as "scheduled_at!",
               started_at, completed_at, timeout_seconds, retry_policy, version as "version!"
        FROM activity_tasks 
        WHERE status = ?
        ORDER BY scheduled_at DESC 
        LIMIT ? OFFSET ?
        "#,
        status,
        limit,
        offset
    )
    .fetch_all(executor)
    .await?;

    Ok(tasks)
}

// 部分更新任务（安全动态SQL实现）
pub async fn update_task<'e, E>(
    executor: E,
    task_token: &str,
    changes: &UpdateActivityTask,
) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let mut query = QueryBuilder::new("UPDATE activity_tasks SET ");
    let mut fields_updated = 0;

    macro_rules! set_field {
        ($field:ident) => {
            if let Some(value) = &changes.$field {
                if fields_updated > 0 {
                    query.push(", ");
                }
                query.push(stringify!($field)).push(" = ").push_bind(value);
                fields_updated += 1;
            }
        };
    }

    set_field!(state_name);
    set_field!(input);
    set_field!(result);
    set_field!(status);
    set_field!(error);
    set_field!(error_details);
    set_field!(attempt);
    set_field!(heartbeat_at);
    set_field!(started_at);
    set_field!(completed_at);
    set_field!(version);

    if fields_updated == 0 {
        return Ok(());
    }

    query.push(" WHERE task_token = ").push_bind(task_token);
    query.build().execute(executor).await?;
    Ok(())
}

// 删除任务记录
pub async fn delete_task<'e, E>(executor: E, task_token: &str) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("DELETE FROM activity_tasks WHERE task_token = ?", task_token)
        .execute(executor)
        .await?;
    Ok(())
}
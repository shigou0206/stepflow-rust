use chrono::Utc;
use sqlx::{Executor, Result, Sqlite};
use crate::models::workflow_event::WorkflowEvent;

// 创建新事件记录，返回生成的自增ID
pub async fn create_event<'e, E>(executor: E, event: &WorkflowEvent) -> Result<i64>
where
    E: Executor<'e, Database = Sqlite>,
{
    let result = sqlx::query!(
        r#"
        INSERT INTO workflow_events (
            run_id, shard_id, event_id, event_type, state_id, state_type, trace_id,
            parent_event_id, context_version, attributes, attr_version, timestamp, archived
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        event.run_id,
        event.shard_id,
        event.event_id,
        event.event_type,
        event.state_id,
        event.state_type,
        event.trace_id,
        event.parent_event_id,
        event.context_version,
        event.attributes,
        event.attr_version,
        event.timestamp,
        event.archived
    )
    .execute(executor)
    .await?;

    Ok(result.last_insert_rowid())
}

// 安全查询单个事件，返回Option防止panic
pub async fn get_event<'e, E>(executor: E, id: i64) -> Result<Option<WorkflowEvent>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let event = sqlx::query_as!(
        WorkflowEvent,
        r#"
        SELECT 
            id as "id!",
            run_id as "run_id!",
            shard_id as "shard_id!",
            event_id as "event_id!",
            event_type as "event_type!",
            state_id, state_type, trace_id, parent_event_id, context_version,
            attributes, attr_version as "attr_version!",
            timestamp as "timestamp!",
            archived as "archived!"
        FROM workflow_events WHERE id = ?
        "#,
        id
    )
    .fetch_optional(executor)
    .await?;

    Ok(event)
}

// 查询指定run_id的所有事件 (分页)
pub async fn find_events_by_run_id<'e, E>(
    executor: E,
    run_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<WorkflowEvent>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let events = sqlx::query_as!(
        WorkflowEvent,
        r#"
        SELECT 
            id as "id!",
            run_id as "run_id!",
            shard_id as "shard_id!",
            event_id as "event_id!",
            event_type as "event_type!",
            state_id, state_type, trace_id, parent_event_id, context_version,
            attributes, attr_version as "attr_version!",
            timestamp as "timestamp!",
            archived as "archived!"
        FROM workflow_events
        WHERE run_id = ?
        ORDER BY event_id ASC
        LIMIT ? OFFSET ?
        "#,
        run_id,
        limit,
        offset
    )
    .fetch_all(executor)
    .await?;

    Ok(events)
}

// 标记事件为归档，并更新归档时间戳
pub async fn archive_event<'e, E>(executor: E, id: i64) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let timestamp = Utc::now().naive_utc();

    sqlx::query!(
        "UPDATE workflow_events SET archived = TRUE, timestamp = ? WHERE id = ?",
        timestamp,
        id
    )
    .execute(executor)
    .await?;

    Ok(())
}

// 按ID删除事件 (谨慎使用)
pub async fn delete_event<'e, E>(executor: E, id: i64) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("DELETE FROM workflow_events WHERE id = ?", id)
        .execute(executor)
        .await?;

    Ok(())
}

// 按run_id批量删除事件 (谨慎使用)
pub async fn delete_events_by_run_id<'e, E>(executor: E, run_id: &str) -> Result<u64>
where
    E: Executor<'e, Database = Sqlite>,
{
    let result = sqlx::query!("DELETE FROM workflow_events WHERE run_id = ?", run_id)
        .execute(executor)
        .await?;

    Ok(result.rows_affected())
}
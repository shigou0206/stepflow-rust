use chrono::NaiveDateTime;
use sqlx::{Executor, QueryBuilder, Result, Sqlite};
use crate::models::timer::{Timer, UpdateTimer};

pub async fn create_timer<'e, E>(executor: E, timer: &Timer) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        r#"
        INSERT INTO timers (
            timer_id, run_id, shard_id, fire_at, status, version, state_name
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        timer.timer_id,
        timer.run_id,
        timer.shard_id,
        timer.fire_at,
        timer.status,
        timer.version,
        timer.state_name
    )
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn get_timer<'e, E>(executor: E, timer_id: &str) -> Result<Option<Timer>>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        Timer,
        r#"
        SELECT timer_id as "timer_id!",
               run_id as "run_id!",
               shard_id as "shard_id!",
               fire_at as "fire_at!",
               status as "status!",
               version as "version!",
               state_name as "state_name!"
        FROM timers WHERE timer_id = ?
        "#,
        timer_id
    )
    .fetch_optional(executor)
    .await
}

pub async fn update_timer<'e, E>(executor: E, timer_id: &str, changes: &UpdateTimer) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let mut query = QueryBuilder::new("UPDATE timers SET ");
    let mut has_fields = false;

    macro_rules! set_field {
        ($field:ident) => {
            if let Some(val) = &changes.$field {
                if has_fields {
                    query.push(", ");
                }
                query.push(stringify!($field)).push(" = ").push_bind(val);
                has_fields = true;
            }
        };
    }

    set_field!(fire_at);
    set_field!(status);
    set_field!(version);
    set_field!(state_name);

    if !has_fields {
        return Ok(());
    }

    query.push(" WHERE timer_id = ").push_bind(timer_id);
    query.build().execute(executor).await?;
    Ok(())
}

pub async fn delete_timer<'e, E>(executor: E, timer_id: &str) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("DELETE FROM timers WHERE timer_id = ?", timer_id)
        .execute(executor)
        .await?;
    Ok(())
}

pub async fn find_timers_before<'e, E>(
    executor: E,
    before: NaiveDateTime,
    limit: i64,
) -> Result<Vec<Timer>>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        Timer,
        r#"
        SELECT timer_id as "timer_id!",
               run_id as "run_id!",
               shard_id as "shard_id!",
               fire_at as "fire_at!",
               status as "status!",
               version as "version!",
               state_name as "state_name!"
        FROM timers 
        WHERE fire_at <= ? 
        ORDER BY fire_at ASC
        LIMIT ?
        "#,
        before,
        limit
    )
    .fetch_all(executor)
    .await
}
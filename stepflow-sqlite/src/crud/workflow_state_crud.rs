use chrono::Utc;
use sqlx::{Executor, Sqlite, Result, QueryBuilder};
use crate::models::workflow_state::{WorkflowState, UpdateWorkflowState};

// 创建新状态
pub async fn create_state<'e, E>(executor: E, state: &WorkflowState) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    tracing::debug!("INSERT state_id={} run_id={}", state.state_id, state.run_id);
    sqlx::query!(
        r#"
        INSERT INTO workflow_states (
            state_id, run_id, shard_id, state_name, state_type, status, input, output,
            error, error_details, started_at, completed_at, created_at, updated_at, version
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        state.state_id, state.run_id, state.shard_id, state.state_name, state.state_type,
        state.status, state.input, state.output, state.error, state.error_details,
        state.started_at, state.completed_at, state.created_at, state.updated_at, state.version
    )
    .execute(executor)
    .await?;
    Ok(())
}

// 安全查询单条状态记录 (避免panic)
pub async fn get_state<'e, E>(executor: E, state_id: &str) -> Result<Option<WorkflowState>>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        WorkflowState,
        r#"
        SELECT 
            state_id as "state_id!",
            run_id as "run_id!",
            shard_id as "shard_id!",
            state_name as "state_name!",
            state_type as "state_type!",
            status as "status!",
            input, output, error, error_details,
            started_at, completed_at,
            created_at as "created_at!",
            updated_at as "updated_at!",
            version as "version!"
        FROM workflow_states WHERE state_id = ?
        "#,
        state_id
    )
    .fetch_optional(executor)
    .await
}

// 根据run_id分页查询状态记录
pub async fn find_states_by_run_id<'e, E>(executor: E, run_id: &str, limit: i64, offset: i64) -> Result<Vec<WorkflowState>>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        WorkflowState,
        r#"
        SELECT 
            state_id as "state_id!",
            run_id as "run_id!",
            shard_id as "shard_id!",
            state_name as "state_name!",
            state_type as "state_type!",
            status as "status!",
            input, output, error, error_details,
            started_at, completed_at,
            created_at as "created_at!",
            updated_at as "updated_at!",
            version as "version!"
        FROM workflow_states WHERE run_id = ? ORDER BY created_at ASC LIMIT ? OFFSET ?
        "#,
        run_id, limit, offset
    )
    .fetch_all(executor)
    .await
}

pub async fn update_state<'e, E>(
    executor: E,
    state_id: &str,
    changes: &UpdateWorkflowState,
) -> Result<()>
where
    E: Executor<'e, Database = Sqlite> + Clone,
{
    // ① 先确认该 state 存在；不存在就报错，让调用方先 create_state
    if get_state(executor.clone(), state_id).await?.is_none() {
        // 用 sqlx 的 RowNotFound 最合适，外层可 map_err
        return Err(sqlx::Error::RowNotFound.into());
    }

    // ② 动态拼 SET 子句
    use sqlx::QueryBuilder;
    let mut qb = QueryBuilder::new("UPDATE workflow_states SET ");
    let mut first = true;

    macro_rules! set_opt {
        ($field:ident) => {
            if let Some(ref val) = changes.$field {
                if !first { qb.push(", "); }
                qb.push(concat!(stringify!($field), " = ")).push_bind(val);
                first = false;
            }
        };
    }

    set_opt!(state_name);
    set_opt!(state_type);
    set_opt!(status);
    set_opt!(input);
    set_opt!(output);
    set_opt!(error);
    set_opt!(error_details);
    set_opt!(started_at);
    set_opt!(completed_at);
    set_opt!(version);

    // 如果什么都没改，就返回
    if first {
        return Ok(());
    }

    // ③ 总是更新时间戳
    qb.push(", updated_at = ").push_bind(Utc::now().naive_utc());

    // ④ WHERE
    qb.push(" WHERE state_id = ").push_bind(state_id);

    qb.build().execute(executor).await?;
    Ok(())
}

// 删除状态记录
pub async fn delete_state<'e, E>(executor: E, state_id: &str) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        "DELETE FROM workflow_states WHERE state_id = ?",
        state_id
    )
    .execute(executor)
    .await?;
    Ok(())
}
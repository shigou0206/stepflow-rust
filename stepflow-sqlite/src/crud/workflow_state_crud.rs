use chrono::Utc;
use sqlx::{Executor, Sqlite, Result, QueryBuilder};
use crate::models::workflow_state::{WorkflowState, UpdateWorkflowState};

// 创建新状态
pub async fn create_state<'e, E>(executor: E, state: &WorkflowState) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
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

// 部分更新状态（避免空更新）
pub async fn update_state<'e, E>(executor: E, state_id: &str, changes: &UpdateWorkflowState) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let mut query = QueryBuilder::new("UPDATE workflow_states SET ");
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

    set_field!(state_name);
    set_field!(state_type);
    set_field!(status);
    set_field!(input);
    set_field!(output);
    set_field!(error);
    set_field!(error_details);
    set_field!(started_at);
    set_field!(completed_at);
    set_field!(version);

    if !has_fields {
        return Ok(()); // 无需更新
    }

    query.push(", updated_at = ").push_bind(Utc::now().naive_utc());
    query.push(" WHERE state_id = ").push_bind(state_id);

    query.build().execute(executor).await?;
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
use sqlx::{Executor, QueryBuilder, Result, Sqlite};
use crate::models::workflow_execution::{WorkflowExecution, UpdateWorkflowExecution};

// 创建记录
pub async fn create_execution<'e, E>(executor: E, exec: &WorkflowExecution) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        r#"
        INSERT INTO workflow_executions (
            run_id, workflow_id, shard_id, template_id, mode,
            current_state_name, status, workflow_type, input, input_version,
            result, result_version, start_time, close_time,
            current_event_id, memo, search_attrs, context_snapshot, version,
            parent_run_id, parent_state_name, dsl_definition
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        exec.run_id,
        exec.workflow_id,
        exec.shard_id,
        exec.template_id,
        exec.mode,
        exec.current_state_name,
        exec.status,
        exec.workflow_type,
        exec.input,
        exec.input_version,
        exec.result,
        exec.result_version,
        exec.start_time,
        exec.close_time,
        exec.current_event_id,
        exec.memo,
        exec.search_attrs,
        exec.context_snapshot,
        exec.version,
        exec.parent_run_id,
        exec.parent_state_name,
        exec.dsl_definition
    )
    .execute(executor)
    .await?;
    Ok(())
}

// 主键查询
pub async fn get_execution<'e, E>(executor: E, run_id: &str) -> Result<Option<WorkflowExecution>>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        WorkflowExecution,
        r#"
        SELECT 
            run_id as "run_id!", workflow_id, shard_id as "shard_id!", template_id,
            mode as "mode!", current_state_name, status as "status!",
            workflow_type as "workflow_type!", input, input_version as "input_version!",
            result, result_version as "result_version!", start_time as "start_time!",
            close_time, current_event_id as "current_event_id!", memo,
            search_attrs, context_snapshot, version as "version!",
            parent_run_id, parent_state_name, dsl_definition
        FROM workflow_executions
        WHERE run_id = ?
        "#,
        run_id
    )
    .fetch_optional(executor)
    .await
}

// 分页查询
pub async fn find_executions<'e, E>(executor: E, limit: i64, offset: i64) -> Result<Vec<WorkflowExecution>>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        WorkflowExecution,
        r#"
        SELECT 
            run_id as "run_id!", workflow_id, shard_id as "shard_id!", template_id,
            mode as "mode!", current_state_name, status as "status!",
            workflow_type as "workflow_type!", input, input_version as "input_version!",
            result, result_version as "result_version!", start_time as "start_time!",
            close_time, current_event_id as "current_event_id!", memo,
            search_attrs, context_snapshot, version as "version!",
            parent_run_id, parent_state_name, dsl_definition
        FROM workflow_executions
        ORDER BY start_time DESC LIMIT ? OFFSET ?
        "#,
        limit,
        offset
    )
    .fetch_all(executor)
    .await
}

// 状态筛选查询
pub async fn find_executions_by_status<'e, E>(executor: E, status: &str, limit: i64, offset: i64) -> Result<Vec<WorkflowExecution>>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        WorkflowExecution,
        r#"
        SELECT 
            run_id as "run_id!", workflow_id, shard_id as "shard_id!", template_id,
            mode as "mode!", current_state_name, status as "status!",
            workflow_type as "workflow_type!", input, input_version as "input_version!",
            result, result_version as "result_version!", start_time as "start_time!",
            close_time, current_event_id as "current_event_id!", memo,
            search_attrs, context_snapshot, version as "version!",
            parent_run_id, parent_state_name, dsl_definition
        FROM workflow_executions
        WHERE status = ? ORDER BY start_time DESC LIMIT ? OFFSET ?
        "#,
        status,
        limit,
        offset
    )
    .fetch_all(executor)
    .await
}

// 安全动态更新
pub async fn update_execution<'e, E>(executor: E, run_id: &str, changes: &UpdateWorkflowExecution) -> Result<()> 
where
    E: Executor<'e, Database = Sqlite>,
{
    let mut query = QueryBuilder::new("UPDATE workflow_executions SET ");
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

    set_field!(workflow_id);
    set_field!(shard_id);
    set_field!(template_id);
    set_field!(mode);
    set_field!(current_state_name);
    set_field!(status);
    set_field!(workflow_type);
    set_field!(input);
    set_field!(input_version);
    set_field!(result);
    set_field!(result_version);
    set_field!(start_time);
    set_field!(close_time);
    set_field!(current_event_id);
    set_field!(memo);
    set_field!(search_attrs);
    set_field!(context_snapshot);
    set_field!(version);
    set_field!(parent_run_id);
    set_field!(parent_state_name);
    set_field!(dsl_definition);

    if !has_fields { return Ok(()); }

    query.push(" WHERE run_id = ").push_bind(run_id);
    query.build().execute(executor).await?;
    Ok(())
}

// 删除
pub async fn delete_execution<'e, E>(executor: E, run_id: &str) -> Result<()> 
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("DELETE FROM workflow_executions WHERE run_id = ?", run_id)
        .execute(executor)
        .await?;
    Ok(())
}

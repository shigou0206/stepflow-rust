use sqlx::{Executor, Sqlite, Result, QueryBuilder};
use crate::models::workflow_visibility::{WorkflowVisibility, UpdateWorkflowVisibility};

// 创建 visibility 记录
pub async fn create_visibility<'e, E>(executor: E, vis: &WorkflowVisibility) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        r#"
        INSERT INTO workflow_visibility (
            run_id, workflow_id, workflow_type, start_time, close_time,
            status, memo, search_attrs, version
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        vis.run_id, vis.workflow_id, vis.workflow_type,
        vis.start_time, vis.close_time,
        vis.status, vis.memo, vis.search_attrs, vis.version
    )
    .execute(executor).await?;
    Ok(())
}

// 根据 run_id 查询 visibility 记录
pub async fn get_visibility<'e, E>(executor: E, run_id: &str) -> Result<Option<WorkflowVisibility>>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        WorkflowVisibility,
        r#"
        SELECT run_id as "run_id!", workflow_id, workflow_type, start_time, close_time, 
               status, memo, search_attrs, version as "version!"
        FROM workflow_visibility WHERE run_id = ?
        "#,
        run_id
    )
    .fetch_optional(executor).await
}

// 根据状态分页查询 visibility
pub async fn find_visibilities_by_status<'e, E>(
    executor: E, status: &str, limit: i64, offset: i64
) -> Result<Vec<WorkflowVisibility>>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_as!(
        WorkflowVisibility,
        r#"
        SELECT run_id as "run_id!", workflow_id, workflow_type, start_time, close_time, 
               status, memo, search_attrs, version as "version!"
        FROM workflow_visibility
        WHERE status = ?
        ORDER BY start_time DESC
        LIMIT ? OFFSET ?
        "#,
        status, limit, offset
    )
    .fetch_all(executor).await
}

// 动态更新 visibility 记录
pub async fn update_visibility<'e, E>(
    executor: E, run_id: &str, changes: &UpdateWorkflowVisibility
) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let mut query = QueryBuilder::new("UPDATE workflow_visibility SET ");
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
    set_field!(workflow_type);
    set_field!(start_time);
    set_field!(close_time);
    set_field!(status);
    set_field!(memo);
    set_field!(search_attrs);
    set_field!(version);

    if !has_fields {
        return Ok(());
    }

    query.push(" WHERE run_id = ").push_bind(run_id);

    query.build().execute(executor).await?;
    Ok(())
}

// 根据 run_id 删除 visibility 记录
pub async fn delete_visibility<'e, E>(executor: E, run_id: &str) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("DELETE FROM workflow_visibility WHERE run_id = ?", run_id)
        .execute(executor).await?;
    Ok(())
}
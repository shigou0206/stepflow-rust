use chrono::Utc;
use sqlx::{Executor, QueryBuilder, Result, Sqlite};
use crate::models::workflow_template::{WorkflowTemplate, UpdateWorkflowTemplate};

// 创建新模板
pub async fn create_template<'e, E>(executor: E, tpl: &WorkflowTemplate) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        r#"
        INSERT INTO workflow_templates
        (template_id, name, description, dsl_definition, version, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        tpl.template_id,
        tpl.name,
        tpl.description,
        tpl.dsl_definition,
        tpl.version,
        tpl.created_at,
        tpl.updated_at
    )
    .execute(executor)
    .await?;
    Ok(())
}

// 主键查询单条记录（推荐安全写法）
pub async fn get_template<'e, E>(executor: E, template_id: &str) -> Result<Option<WorkflowTemplate>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let result = sqlx::query_as!(
        WorkflowTemplate,
        r#"
        SELECT template_id as "template_id!", 
               name as "name!", 
               description,
               dsl_definition as "dsl_definition!", 
               version as "version!", 
               created_at as "created_at!", 
               updated_at as "updated_at!"
        FROM workflow_templates WHERE template_id = ?"#,
        template_id
    )
    .fetch_optional(executor)
    .await?;

    Ok(result)
}

// 分页查询模板列表
pub async fn find_templates<'e, E>(executor: E, limit: i64, offset: i64) -> Result<Vec<WorkflowTemplate>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let templates = sqlx::query_as!(
        WorkflowTemplate,
        r#"
        SELECT template_id as "template_id!", 
               name as "name!", 
               description,
               dsl_definition as "dsl_definition!", 
               version as "version!", 
               created_at as "created_at!", 
               updated_at as "updated_at!"
        FROM workflow_templates 
        ORDER BY created_at DESC 
        LIMIT ? OFFSET ?"#,
        limit,
        offset
    )
    .fetch_all(executor)
    .await?;

    Ok(templates)
}

// 部分更新模板
pub async fn update_template<'e, E>(
    executor: E,
    template_id: &str,
    changes: &UpdateWorkflowTemplate,
) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    let mut query = QueryBuilder::new("UPDATE workflow_templates SET ");
    let mut has_prev = false;

    macro_rules! set_field {
        ($field:ident) => {
            if let Some(value) = &changes.$field {
                if has_prev {
                    query.push(", ");
                }
                query.push(stringify!($field)).push(" = ").push_bind(value);
                has_prev = true;
            }
        };
    }

    set_field!(name);
    set_field!(description);
    set_field!(dsl_definition);
    set_field!(version);

    if has_prev {
        query.push(", updated_at = ").push_bind(Utc::now().naive_utc());
    } else {
        return Ok(()); // 若无任何更新字段，直接返回成功
    }

    query
        .push(" WHERE template_id = ")
        .push_bind(template_id);

    query.build().execute(executor).await?;

    Ok(())
}

// 删除模板
pub async fn delete_template<'e, E>(executor: E, template_id: &str) -> Result<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("DELETE FROM workflow_templates WHERE template_id = ?", template_id)
        .execute(executor)
        .await?;
    Ok(())
}
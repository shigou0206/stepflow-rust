use async_trait::async_trait;
use stepflow_storage::PersistenceManager;
use stepflow_sqlite::models::workflow_template::{WorkflowTemplate, UpdateWorkflowTemplate};
use crate::dto::template::*;
use crate::error::{AppResult, AppError};
use anyhow::Context;

#[derive(Clone)]
pub struct TemplateSqlxSvc {
    pm: std::sync::Arc<dyn PersistenceManager>,
}
impl TemplateSqlxSvc {
    pub fn new(pm: std::sync::Arc<dyn PersistenceManager>) -> Self { Self { pm } }

    async fn insert_or_update(
        &self,
        id: &str,
        body: TemplateUpsert,
        is_create: bool,
    ) -> AppResult<TemplateDto> {
        if is_create {
            let row = WorkflowTemplate {
                template_id: id.to_string(),
                name: body.name.clone(),
                description: None,
                dsl_definition: serde_json::to_string(&body.dsl).context("序列化 DSL 失败")?,
                version: 1,
                created_at: chrono::Utc::now().naive_utc(),
                updated_at: chrono::Utc::now().naive_utc(),
            };
            self.pm.create_template(&row).await?;
            Ok(TemplateDto {
                id: row.template_id,
                name: row.name,
                dsl: body.dsl,
                created_at: chrono::DateTime::from_utc(row.created_at, chrono::Utc),
            })
        } else {
            // 若不存在可决定返回 404 或新增，这里选择 404
            if self.pm.get_template(id).await?.is_none() {
                return Err(AppError::NotFound);
            }
            let changes = UpdateWorkflowTemplate {
                name: Some(body.name),
                description: None,
                dsl_definition: Some(serde_json::to_string(&body.dsl).context("序列化 DSL 失败")?),
                version: Some(1),
            };
            self.pm.update_template(id, &changes).await?;
            
            // 重新获取更新后的数据
            let row = self.pm.get_template(id).await?.unwrap();
            Ok(TemplateDto {
                id: row.template_id,
                name: row.name,
                dsl: serde_json::from_str(&row.dsl_definition).unwrap_or_default(),
                created_at: chrono::DateTime::from_utc(row.created_at, chrono::Utc),
            })
        }
    }
}

#[async_trait]
impl crate::service::TemplateService for TemplateSqlxSvc {
    async fn create(&self, body: TemplateUpsert) -> AppResult<TemplateDto> {
        let id = uuid::Uuid::new_v4().to_string();
        self.insert_or_update(&id, body, true).await
    }

    async fn update(&self, id:&str, body:TemplateUpsert) -> AppResult<TemplateDto> {
        self.insert_or_update(id, body, false).await
    }

    async fn get(&self, id:&str) -> AppResult<TemplateDto> {
        let row = self.pm.get_template(id).await?
            .ok_or(AppError::NotFound)?;
        Ok(TemplateDto {
            id: row.template_id,
            name: row.name,
            dsl: serde_json::from_str(&row.dsl_definition).unwrap_or_default(),
            created_at: chrono::DateTime::from_utc(row.created_at, chrono::Utc),
        })
    }

    async fn list(&self) -> AppResult<Vec<TemplateDto>> {
        let rows = self.pm.find_templates(100, 0).await?;
        Ok(rows.into_iter().map(|r| TemplateDto {
            id: r.template_id,
            name: r.name,
            dsl: serde_json::from_str(&r.dsl_definition).unwrap_or_default(),
            created_at: chrono::DateTime::from_utc(r.created_at, chrono::Utc),
        }).collect())
    }

    async fn delete(&self, id:&str) -> AppResult<()> {
        self.pm.delete_template(id).await?;
        Ok(())
    }
}
pub mod template;

use crate::error::AppResult;
use async_trait::async_trait;
use crate::dto::template::*;

#[async_trait]
pub trait TemplateService: Clone + Send + Sync + 'static {
    async fn create(&self, dto: TemplateUpsert)               -> AppResult<TemplateDto>;
    async fn update(&self, id: &str, dto: TemplateUpsert)     -> AppResult<TemplateDto>;
    async fn get   (&self, id: &str)                          -> AppResult<TemplateDto>;
    async fn list  (&self)                                    -> AppResult<Vec<TemplateDto>>;
    async fn delete(&self, id: &str)                          -> AppResult<()>;
}

pub use template::TemplateSqlxSvc as TemplateSvc;

// gateway/src/service/mod.rs
pub mod execution;
pub use execution::ExecutionSqlxSvc as ExecutionSvc;
use crate::dto::execution::*;

#[async_trait]
pub trait ExecutionService: Clone + Send + Sync + 'static {
    async fn start(&self, req: ExecStart) -> AppResult<ExecDto>;
    async fn get  (&self, run_id: &str) -> AppResult<ExecDto>;
    async fn list (&self, limit: i64, offset: i64) -> AppResult<Vec<ExecDto>>;
}
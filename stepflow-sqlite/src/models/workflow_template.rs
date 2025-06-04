use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkflowTemplate {
    pub template_id: String,
    pub name: String,
    pub description: Option<String>,
    pub dsl_definition: String,
    pub version: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateWorkflowTemplate {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub dsl_definition: Option<String>,
    pub version: Option<i64>,
}

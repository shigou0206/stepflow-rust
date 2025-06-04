use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredWorkflowTemplate {
    pub template_id: String,
    pub name: String,
    pub description: Option<String>,
    pub dsl_definition: String,
    pub version: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateStoredWorkflowTemplate {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub dsl_definition: Option<String>,
    pub version: Option<i64>,
} 
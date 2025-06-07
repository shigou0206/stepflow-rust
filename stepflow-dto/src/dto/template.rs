use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct TemplateUpsert {
    pub name: String,
    pub dsl:  Value,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct TemplateDto {
    pub id:   String,
    pub name: String,
    pub dsl:  Value,
    pub created_at: DateTime<Utc>,
}

impl From<TemplateUpsert> for TemplateDto {
    fn from(u: TemplateUpsert) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: u.name,
            dsl: u.dsl,
            created_at: chrono::Utc::now(),
        }
    }
}
// gateway/src/dto/execution.rs
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExecStart {
    #[schema(example = "INLINE")]
    pub mode: String,                 // "INLINE" | "DEFERRED"
    pub template_id: Option<String>,  // 二选一
    pub dsl:         Option<Value>,
    pub init_ctx:    Option<Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExecDto {
    pub run_id:  String,
    pub mode:    String,
    pub status:  String,
    pub result:  Option<Value>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}


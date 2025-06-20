// ✅ gateway/src/dto/execution.rs：补充子流程启动字段（parent_run_id, parent_state_name）
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExecStart {
    // 启动主流程的方式（模板 or DSL）
    pub template_id: Option<String>,
    pub dsl:         Option<Value>,
    pub init_ctx:    Option<Value>,

    // 🔽 新增字段：支持作为子流程启动
    pub run_id: Option<String>,
    pub parent_run_id: Option<String>,
    pub parent_state_name: Option<String>,
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

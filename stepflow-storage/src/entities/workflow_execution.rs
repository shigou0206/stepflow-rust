use chrono::NaiveDateTime;
use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredWorkflowExecution {
    pub run_id: String,
    pub workflow_id: Option<String>,
    pub shard_id: i64,
    pub template_id: Option<String>,
    pub mode: String,
    pub current_state_name: Option<String>,
    pub status: String,
    pub workflow_type: String,
    pub input: Option<Value>,
    pub input_version: i64,
    pub result: Option<Value>,
    pub result_version: i64,
    pub start_time: NaiveDateTime,
    pub close_time: Option<NaiveDateTime>,
    pub current_event_id: i64,
    pub memo: Option<String>,
    pub search_attrs: Option<Value>,
    pub context_snapshot: Option<Value>,
    pub version: i64,

    // 新增字段：子流程支持
    pub parent_run_id: Option<String>,
    pub parent_state_name: Option<String>,
    pub dsl_definition: Option<Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateStoredWorkflowExecution {
    pub workflow_id: Option<Option<String>>,
    pub shard_id: Option<i64>,
    pub template_id: Option<Option<String>>,
    pub mode: Option<String>,
    pub current_state_name: Option<Option<String>>,
    pub status: Option<String>,
    pub workflow_type: Option<String>,
    pub input: Option<Option<Value>>,
    pub input_version: Option<i64>,
    pub result: Option<Option<Value>>,
    pub result_version: Option<i64>,
    pub start_time: Option<NaiveDateTime>,
    pub close_time: Option<Option<NaiveDateTime>>,
    pub current_event_id: Option<i64>,
    pub memo: Option<Option<String>>,
    pub search_attrs: Option<Option<Value>>,
    pub context_snapshot: Option<Option<Value>>,
    pub version: Option<i64>,

    // 新增字段：子流程支持
    pub parent_run_id: Option<Option<String>>,
    pub parent_state_name: Option<Option<String>>,
    pub dsl_definition: Option<Option<Value>>,
}
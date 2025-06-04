use chrono::NaiveDateTime;
use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoredWorkflowVisibility {
    pub run_id: String,
    pub workflow_id: Option<String>,
    pub workflow_type: Option<String>,
    pub start_time: Option<NaiveDateTime>,
    pub close_time: Option<NaiveDateTime>,
    pub status: Option<String>,
    pub memo: Option<String>,
    pub search_attrs: Option<Value>,  // 使用 Value 以支持结构化搜索属性
    pub version: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateStoredWorkflowVisibility {
    pub workflow_id: Option<Option<String>>,
    pub workflow_type: Option<Option<String>>,
    pub start_time: Option<Option<NaiveDateTime>>,
    pub close_time: Option<Option<NaiveDateTime>>,
    pub status: Option<Option<String>>,
    pub memo: Option<Option<String>>,
    pub search_attrs: Option<Option<Value>>,  // 使用 Value
    pub version: Option<i64>,
} 
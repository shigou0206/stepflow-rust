use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 精简版 JSON Schema，用于前端动态控件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingSchema {
    pub type_: String,                 // string/number/boolean/array/object
    pub enum_: Option<Vec<Value>>,
    pub description: Option<String>,
    pub default: Option<Value>,
}
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 每条规则的执行快照
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingStepSnapshot {
    pub key: String,
    pub success: bool,
    pub error: Option<String>,
    pub output: Option<Value>,
}

/// 引擎最终返回
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingResult {
    pub output: Value,
    pub steps: Vec<MappingStepSnapshot>,
}
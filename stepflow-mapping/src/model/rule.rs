use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 支持的映射类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MappingType {
    JsonPath,
    Constant,
    Expr,
    Template,
    SubMapping,
    FormField,
}

/// 字段合并策略
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MergeStrategy {
    Overwrite,
    Ignore,
    Append,
    Merge,
}

impl Default for MergeStrategy {
    fn default() -> Self {
        MergeStrategy::Overwrite
    }
}

/// 单条字段映射规则
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MappingRule {
    pub key: String,

    #[serde(rename = "type")]
    pub mapping_type: MappingType,

    // —— 各类型专属字段 ——
    pub source: Option<String>,      // jsonpath
    pub transform: Option<String>,   // expr
    pub template: Option<String>,    // template
    pub value: Option<Value>,        // constant
    pub sub_mappings: Option<Vec<MappingRule>>, // sub_mapping

    // —— 通用字段 ——
    #[serde(default)]
    pub merge_strategy: MergeStrategy,

    pub condition: Option<String>,      // future
    pub depends_on: Option<Vec<String>>,

    // —— UI / 文档辅助 ——
    pub comment: Option<String>,
    pub lang: Option<String>,
    pub expected_type: Option<String>,
    pub schema: Option<Value>,
}

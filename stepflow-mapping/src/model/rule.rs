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

impl Default for MappingRule {
    fn default() -> Self {
        Self {
            key: "".to_string(),
            mapping_type: MappingType::Constant,
            source: None,
            transform: None,
            template: None,
            value: None,
            sub_mappings: None,
            merge_strategy: MergeStrategy::Overwrite,
            condition: None,
            depends_on: None,
            comment: None,
            lang: None,
            expected_type: None,
            schema: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mapping_rule_serde_constant() {
        let rule_json = json!({
            "key": "foo",
            "type": "constant",
            "value": 123,
            "mergeStrategy": "overwrite"
        });
        let rule: MappingRule = serde_json::from_value(rule_json.clone()).unwrap();
        assert_eq!(rule.key, "foo");
        assert_eq!(rule.mapping_type, MappingType::Constant);
        assert_eq!(rule.value, Some(json!(123)));
        assert_eq!(rule.merge_strategy, MergeStrategy::Overwrite);
        let out = serde_json::to_value(&rule).unwrap();
        assert_eq!(out["type"], "constant");
        assert_eq!(out["mergeStrategy"], "overwrite");
    }

    #[test]
    fn test_merge_strategy_serde() {
        let s = serde_json::to_string(&MergeStrategy::Append).unwrap();
        assert_eq!(s, "\"append\"");
        let strat: MergeStrategy = serde_json::from_str(&s).unwrap();
        assert_eq!(strat, MergeStrategy::Append);
    }

    #[test]
    fn test_mapping_rule_with_depends_on() {
        let rule_json = json!({
            "key": "bar",
            "type": "jsonPath",
            "source": "$.foo",
            "dependsOn": ["foo"]
        });
        let rule: MappingRule = serde_json::from_value(rule_json).unwrap();
        assert_eq!(rule.key, "bar");
        assert_eq!(rule.mapping_type, MappingType::JsonPath);
        assert_eq!(rule.source.as_deref(), Some("$.foo"));
        assert_eq!(rule.depends_on.as_ref().unwrap(), &["foo"]);
    }
}

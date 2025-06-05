pub mod constant;
pub mod jsonpath;
pub mod expr;
pub mod template;
pub mod submapping;
pub mod form_field;

use crate::{
    error::{MappingError, Result},
    model::rule::{MappingRule, MappingType},
};
use serde_json::Value;

/// 按照 MappingType 分发到具体解析器
pub fn resolve(rule: &MappingRule, input: &Value) -> Result<Value> {
    match rule.mapping_type {
        MappingType::Constant   => constant::resolve_constant(rule),
        MappingType::JsonPath   => jsonpath::resolve_jsonpath(rule, input),
        MappingType::Expr       => expr::resolve_expr(rule, input),
        MappingType::Template   => template::resolve_template(rule, input),
        MappingType::SubMapping => submapping::resolve_submapping(rule, input),
        _ => Err(MappingError::UnsupportedType(format!("{:?}", rule.mapping_type))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::rule::{MappingRule, MappingType};
    use serde_json::json;
    use crate::model::dsl::MappingDSL;

    #[test]
    fn test_resolve_constant() {
        let rule = MappingRule {
            key: "foo".to_string(),
            mapping_type: MappingType::Constant,
            value: Some(json!(123)),
            source: None,
            transform: None,
            template: None,
            sub_mappings: None,
            merge_strategy: Default::default(),
            condition: None,
            depends_on: None,
            comment: None,
            lang: None,
            expected_type: None,
            schema: None,
        };
        let input = json!({});
        let out = resolve(&rule, &input).unwrap();
        assert_eq!(out, json!(123));
    }

    #[test]
    fn test_resolve_jsonpath() {
        let rule = MappingRule {
            key: "bar".to_string(),
            mapping_type: MappingType::JsonPath,
            source: Some("$.foo".to_string()),
            value: None,
            transform: None,
            template: None,
            sub_mappings: None,
            merge_strategy: Default::default(),
            condition: None,
            depends_on: None,
            comment: None,
            lang: None,
            expected_type: None,
            schema: None,
        };
        let input = json!({"foo": 42});
        let out = resolve(&rule, &input).unwrap();
        assert_eq!(out, json!(42));
    }

    #[test]
    fn test_resolve_expr_error() {
        let rule = MappingRule {
            key: "baz".to_string(),
            mapping_type: MappingType::Expr,
            transform: Some("invalid + 1".to_string()),
            value: None,
            source: None,
            template: None,
            sub_mappings: None,
            merge_strategy: Default::default(),
            condition: None,
            depends_on: None,
            comment: None,
            lang: None,
            expected_type: None,
            schema: None,
        };
        let input = json!({"foo": 1});
        let out = resolve(&rule, &input);
        assert!(out.is_err());
    }

    #[test]
    fn test_resolve_submapping() {
        let sub_rule = MappingRule {
            key: "inner".to_string(),
            mapping_type: MappingType::Constant,
            value: Some(json!(100)),
            source: None,
            transform: None,
            template: None,
            sub_mappings: None,
            merge_strategy: Default::default(),
            condition: None,
            depends_on: None,
            comment: None,
            lang: None,
            expected_type: None,
            schema: None,
        };
        let _sub_dsl = MappingDSL {
            mappings: vec![sub_rule],
            ..Default::default()
        };
        let rule = MappingRule {
            key: "sub".to_string(),
            mapping_type: MappingType::SubMapping,
            sub_mappings: Some(vec![
                MappingRule {
                    key: "inner".to_string(),
                    mapping_type: MappingType::Constant,
                    value: Some(json!(100)),
                    source: None,
                    transform: None,
                    template: None,
                    sub_mappings: None,
                    merge_strategy: Default::default(),
                    condition: None,
                    depends_on: None,
                    comment: None,
                    lang: None,
                    expected_type: None,
                    schema: None,
                }
            ]),
            value: None,
            source: Some("$".to_string()),
            transform: None,
            template: None,
            merge_strategy: Default::default(),
            condition: None,
            depends_on: None,
            comment: None,
            lang: None,
            expected_type: None,
            schema: None,
        };
        let input = json!({});
        let out = resolve(&rule, &input).unwrap();
        assert_eq!(out, json!([{"inner": 100}]));
    }
}
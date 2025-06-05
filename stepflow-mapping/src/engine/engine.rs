use serde_json::{Map, Value};

use crate::{
    error::Result,
    engine::context::MappingContext,
    graph::builder::sort_rules,
    model::{
        dsl::{MappingDSL, PreserveFields},
        result::MappingStepSnapshot,
        rule::MappingType,
    },
    resolver,
    utils::merge_value,
};

pub struct MappingEngine;

impl MappingEngine {
    /// 执行映射：支持 Constant / JsonPath / Expr / Template / SubMapping
    /// 并按 `dependsOn` 拓扑排序
    pub fn apply(dsl: MappingDSL, input: &Value) -> Result<Value> {
        // 1️⃣ preserve
        let init = match dsl.preserve {
            PreserveFields::All => input.as_object().cloned().unwrap_or_default(),
            PreserveFields::Some(ref keys) => {
                let mut m = Map::new();
                if let Some(obj) = input.as_object() {
                    for k in keys {
                        if let Some(v) = obj.get(k) {
                            m.insert(k.clone(), v.clone());
                        }
                    }
                }
                m
            }
            PreserveFields::None => Map::new(),
        };
        let mut ctx = MappingContext::new(init);

        // 2️⃣ 拓扑排序后逐条执行
        let ordered = sort_rules(&dsl.mappings)?;
        for rule in ordered {
            // Expr 规则的输入：将原始 input 与已累积 ctx.output 合并
            let resolver_input = if rule.mapping_type == MappingType::Expr {
                let mut merged = input.clone();
                if let (Value::Object(base), Value::Object(overlay)) =
                    (&mut merged, Value::Object(ctx.output.clone()))
                {
                    for (k, v) in overlay {
                        base.insert(k, v);
                    }
                }
                merged
            } else {
                input.clone()
            };

            match resolver::resolve(&rule, &resolver_input) {
                Ok(val) => {
                    merge_value(&mut ctx.output, &rule.key, val.clone(), rule.merge_strategy);
                    ctx.steps.push(MappingStepSnapshot {
                        key: rule.key,
                        success: true,
                        error: None,
                        output: Some(val),
                    });
                }
                Err(err) => {
                    ctx.steps.push(MappingStepSnapshot {
                        key: rule.key,
                        success: false,
                        error: Some(err.to_string()),
                        output: None,
                    });
                }
            }
        }

        // 3️⃣ 返回合并后的 JSON
        Ok(Value::Object(ctx.output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MappingDSL, MappingRule, MappingType, PreserveFields};
    use serde_json::json;

    #[test]
    fn test_apply_constant_rule() {
        let dsl = MappingDSL {
            namespace: Some("test".to_string()),
            version: Some("1.0".to_string()),
            description: None,
            preserve: PreserveFields::None,
            debug: false,
            mappings: vec![
                MappingRule {
                    key: "foo".to_string(),
                    mapping_type: MappingType::Constant,
                    value: Some(json!(42)),
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
            ],
        };
        let input = json!({"bar": 1});
        let result = MappingEngine::apply(dsl, &input).unwrap();
        assert_eq!(result["foo"], 42);
        assert!(result.get("bar").is_none());
    }

    #[test]
    fn test_apply_preserve_all() {
        let dsl = MappingDSL {
            preserve: PreserveFields::All,
            mappings: vec![],
            ..Default::default()
        };
        let input = json!({"a": 1, "b": 2});
        let result = MappingEngine::apply(dsl, &input).unwrap();
        assert_eq!(result["a"], 1);
        assert_eq!(result["b"], 2);
    }

    #[test]
    fn test_apply_merge_strategy_append() {
        let dsl = MappingDSL {
            preserve: PreserveFields::None,
            mappings: vec![
                MappingRule {
                    key: "arr".to_string(),
                    mapping_type: MappingType::Constant,
                    value: Some(json!(1)),
                    merge_strategy: crate::model::rule::MergeStrategy::Append,
                    source: None,
                    transform: None,
                    template: None,
                    sub_mappings: None,
                    condition: None,
                    depends_on: None,
                    comment: None,
                    lang: None,
                    expected_type: None,
                    schema: None,
                },
                MappingRule {
                    key: "arr".to_string(),
                    mapping_type: MappingType::Constant,
                    value: Some(json!(2)),
                    merge_strategy: crate::model::rule::MergeStrategy::Append,
                    source: None,
                    transform: None,
                    template: None,
                    sub_mappings: None,
                    condition: None,
                    depends_on: None,
                    comment: None,
                    lang: None,
                    expected_type: None,
                    schema: None,
                },
            ],
            ..Default::default()
        };
        let input = json!({});
        let result = MappingEngine::apply(dsl, &input).unwrap();
        assert_eq!(result["arr"], json!([1, 2]));
    }
}
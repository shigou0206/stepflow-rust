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
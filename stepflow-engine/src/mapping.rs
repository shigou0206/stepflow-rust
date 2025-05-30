//! Mapping utilities shared by the workflow engine.
//!
//! `MappingPipeline` wraps optional **InputMapping** and **OutputMapping** definitions
//! (both are `stepflow_mapping::MappingDSL`).  The engine调用顺序：
//!
//!   ① `apply_input(ctx)`   —— 在进入具体 handler 前执行
//!   ② 业务 handler         —— 仅关心 exec_in → raw_out
//!   ③ `apply_output(out)` —— handler 返回后执行
//!
//! **当前简化**
//! * 只实现 `MergeStrategy::Overwrite` / `Ignore` 两种浅合并策略
//! * `stepflow_mapping::MappingEngine` 仅需支持 `JsonPath` / `Constant` 两变体
//!
//! 后续若要支持 Append / Merge 深合并、更多映射类型，只需扩展此文件即可。

use serde_json::{Map, Value};
use stepflow_mapping::{MappingDSL, MappingEngine};
use stepflow_mapping::model::rule::MergeStrategy;

/// Lightweight pipeline for a single state (borrowed refs to DSL).
#[derive(Debug, Clone, Copy)]
pub struct MappingPipeline<'a> {
    pub input_mapping:  Option<&'a MappingDSL>,
    pub output_mapping: Option<&'a MappingDSL>,
}

impl<'a> MappingPipeline<'a> {
    /// Apply `input_mapping` to *ctx* (clone if None).
    pub fn apply_input(&self, ctx: &Value) -> Result<Value, String> {
        if let Some(cfg) = self.input_mapping {
            MappingEngine::apply(cfg.clone(), ctx)
                .map_err(|e| format!("InputMapping error: {e}"))
        } else {
            Ok(ctx.clone())
        }
    }

    /// Apply `output_mapping` to *raw_out* then merge with *base_ctx*.
    pub fn apply_output(&self, raw_out: &Value, base_ctx: &Value) -> Result<Value, String> {
        // 1️⃣ 执行 OutputMapping（若有）
        let mapped = if let Some(cfg) = self.output_mapping {
            MappingEngine::apply(cfg.clone(), raw_out)
                .map_err(|e| format!("OutputMapping error: {e}"))?
        } else {
            raw_out.clone()
        };

        // 2️⃣ 根据第 1 条 rule 的 merge_strategy（默认 Overwrite）做浅合并
        let strategy = self
            .output_mapping
            .and_then(|m| m.mappings.first())
            .map(|r| r.merge_strategy)
            .unwrap_or(MergeStrategy::Overwrite);

        Ok(merge_shallow(base_ctx, &mapped, strategy))
    }
}

// -----------------------------------------------------------------------------
// Helper – shallow merge with simple strategy selector
// -----------------------------------------------------------------------------
fn merge_shallow(base: &Value, mapped: &Value, strat: MergeStrategy) -> Value {
    match (base, mapped) {
        (Value::Object(b), Value::Object(m)) => {
            let mut combined: Map<String, Value> = b.clone();
            for (k, v) in m {
                match strat {
                    MergeStrategy::Overwrite => {
                        combined.insert(k.clone(), v.clone());
                    }
                    MergeStrategy::Ignore => {
                        combined.entry(k.clone()).or_insert_with(|| v.clone());
                    }
                    // 其他策略待实现
                    MergeStrategy::Append | MergeStrategy::Merge => {
                        combined.insert(k.clone(), v.clone());
                    }
                }
            }
            Value::Object(combined)
        }
        // 非对象直接根据策略选返回值
        _ => match strat {
            MergeStrategy::Ignore => base.clone(),
            _ => mapped.clone(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_merge_overwrite() {
        let a = json!({ "x": 1, "y": 2 });
        let b = json!({ "y": 99, "z": 3 });
        let r = merge_shallow(&a, &b, MergeStrategy::Overwrite);
        assert_eq!(r, json!({ "x": 1, "y": 99, "z": 3 }));
    }

    #[test]
    fn test_merge_ignore() {
        let a = json!({ "x": 1, "y": 2 });
        let b = json!({ "y": 99, "z": 3 });
        let r = merge_shallow(&a, &b, MergeStrategy::Ignore);
        assert_eq!(r, json!({ "x": 1, "y": 2, "z": 3 }));
    }
}
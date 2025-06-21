use serde_json::{Map, Value};
use stepflow_mapping::model::rule::MergeStrategy;
use stepflow_mapping::{MappingDSL, MappingEngine};

/// Lightweight pipeline for a single state (borrowed refs to DSL).
#[derive(Debug, Clone, Copy)]
pub struct MappingPipeline<'a> {
    pub input_mapping: Option<&'a MappingDSL>,
    pub output_mapping: Option<&'a MappingDSL>,
}

impl<'a> MappingPipeline<'a> {
    /// Apply `input_mapping` to *ctx* (clone if None).
    pub fn apply_input(&self, ctx: &Value) -> Result<Value, String> {
        if let Some(cfg) = self.input_mapping {
            MappingEngine::apply(cfg.clone(), ctx).map_err(|e| format!("InputMapping error: {e}"))
        } else {
            Ok(ctx.clone())
        }
    }

    /// For MapState: inject one item with item_context_key into parent_ctx, then apply mapping.
    pub fn apply_input_for_map_item(
        &self,
        parent_ctx: &Value,
        item_context_key: &str,
        item_value: &Value,
    ) -> Result<Value, String> {
        let mut merged = match parent_ctx {
            Value::Object(map) => map.clone(),
            _ => Map::new(),
        };
        merged.insert(item_context_key.to_string(), item_value.clone());

        let base = Value::Object(merged.clone());

        if let Some(cfg) = self.input_mapping {
            let mapped = MappingEngine::apply(cfg.clone(), &base)
                .map_err(|e| format!("InputMapping error: {e}"))?;
            Ok(merge_shallow(&base, &mapped, MergeStrategy::Overwrite))
        } else {
            Ok(base)
        }
    }

    /// Apply `output_mapping` to *raw_out* then merge with *base_ctx*.
    pub fn apply_output(&self, raw_out: &Value, base_ctx: &Value) -> Result<Value, String> {
        let mapped = if let Some(cfg) = self.output_mapping {
            MappingEngine::apply(cfg.clone(), raw_out)
                .map_err(|e| format!("OutputMapping error: {e}"))?
        } else {
            raw_out.clone()
        };

        let strategy = self
            .output_mapping
            .and_then(|m| m.mappings.first())
            .map(|r| r.merge_strategy)
            .unwrap_or(MergeStrategy::Overwrite);

        Ok(merge_shallow(base_ctx, &mapped, strategy))
    }

    /// Apply `output_mapping` to a list of results and merge all into base_ctx.
    pub fn apply_output_for_map_items(
        &self,
        results: &[Value],
        base_ctx: &Value,
    ) -> Result<Value, String> {
        let mut ctx = base_ctx.clone();
        for result in results {
            ctx = self.apply_output(result, &ctx)?;
        }
        Ok(ctx)
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
                    MergeStrategy::Append | MergeStrategy::Merge => {
                        combined.insert(k.clone(), v.clone());
                    }
                }
            }
            Value::Object(combined)
        }
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
    use serde_yaml;

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

    #[test]
    fn test_apply_input_for_map_item_simple_injection() {
        let base_ctx = json!({ "token": "abc" });
        let item = json!({ "id": 1, "name": "Alice" });
        let pipeline = MappingPipeline {
            input_mapping: None,
            output_mapping: None,
        };

        let injected = pipeline
            .apply_input_for_map_item(&base_ctx, "item", &item)
            .unwrap();

        assert_eq!(
            injected,
            json!({
                "token": "abc",
                "item": { "id": 1, "name": "Alice" }
            })
        );
    }

    #[test]
    fn test_apply_input_for_map_item_with_yaml() {
        use stepflow_mapping::model::dsl::MappingDSL;

        let yaml = r#"
mappings:
  - key: url
    type: constant
    value: https://example.com/api
  - key: body
    type: jsonPath
    source: $.user
"#;

        let dsl: MappingDSL = serde_yaml::from_str(yaml).expect("Failed to parse YAML");

        let base_ctx = json!({ "token": "abc123" });
        let item = json!({ "name": "Alice", "age": 30 });

        let pipeline = MappingPipeline {
            input_mapping: Some(&dsl),
            output_mapping: None,
        };

        let mapped = pipeline
            .apply_input_for_map_item(&base_ctx, "user", &item)
            .unwrap();

        assert_eq!(
            mapped,
            json!({
                "token": "abc123",
                "user": { "name": "Alice", "age": 30 },
                "url": "https://example.com/api",
                "body": { "name": "Alice", "age": 30 }
            })
        );
    }

    #[test]
    fn test_apply_output_for_map_items() {
        let dsl_yaml = r#"
mappings:
  - key: allUsers
    type: jsonPath
    source: $
"#;
        let dsl: MappingDSL = serde_yaml::from_str(dsl_yaml).unwrap();
        let pipeline = MappingPipeline {
            input_mapping: None,
            output_mapping: Some(&dsl),
        };

        let results = vec![
            json!({ "name": "Alice", "age": 30 }),
            json!({ "name": "Bob", "age": 25 }),
        ];

        let base = json!({});
        let merged = pipeline.apply_output_for_map_items(&results, &base).unwrap();

        assert_eq!(
            merged,
            json!({
                "allUsers": { "name": "Bob", "age": 25 } // 最后一个覆盖
            })
        );
    }
}
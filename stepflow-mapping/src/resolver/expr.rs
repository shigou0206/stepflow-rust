#![allow(dead_code)]

use rhai::{Dynamic, Engine, Map as RhaiMap, Scope};
use serde_json::Value;

use crate::{
    error::{MappingError, Result},
    model::rule::MappingRule,
};

/// -------- JSON ➜ Rhai Dynamic --------
fn json_to_dynamic(v: &Value) -> Dynamic {
    match v {
        Value::Null        => Dynamic::UNIT,
        Value::Bool(b)     => Dynamic::from_bool(*b),
        Value::Number(n)   => {
            if let Some(i) = n.as_i64() { Dynamic::from(i) }
            else if let Some(f) = n.as_f64() { Dynamic::from(f) }
            else { Dynamic::UNIT }
        }
        Value::String(s)   => Dynamic::from(s.clone()),
        Value::Array(arr)  => {
            let vec: Vec<Dynamic> = arr.iter().map(json_to_dynamic).collect();
            Dynamic::from_array(vec)
        }
        Value::Object(obj) => {
            let mut map = RhaiMap::new();
            for (k, v) in obj {
                map.insert(k.into(), json_to_dynamic(v));
            }
            Dynamic::from_map(map)
        }
    }
}

/// -------- 解析表达式 --------
pub fn resolve_expr(rule: &MappingRule, input: &Value) -> Result<Value> {
    let expr = rule
        .transform
        .as_ref()
        .ok_or_else(|| MappingError::MissingField("transform"))?;

    let mut scope = Scope::new();
    scope.push("input", json_to_dynamic(input));

    let engine = Engine::new();
    let out = engine
        .eval_with_scope::<Dynamic>(&mut scope, expr)
        .map_err(|e| MappingError::Expr(e.to_string()))?;

    serde_json::to_value(out).map_err(Into::into)
}
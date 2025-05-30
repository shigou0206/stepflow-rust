use jsonpath_lib::select;
use serde_json::Value;

use crate::{
    error::{MappingError, Result},
    model::rule::MappingRule,
};

pub fn resolve_jsonpath(rule: &MappingRule, input: &Value) -> Result<Value> {
    let path = rule
        .source
        .as_ref()
        .ok_or_else(|| MappingError::MissingField("source"))?;

    let hits = select(input, path).map_err(|e| MappingError::JsonPath(e.to_string()))?;

    // 取第一条并克隆为独立 Value；若无命中返回 Null
    let val = hits.get(0).map(|v| (*v).clone()).unwrap_or(Value::Null);

    Ok(val)
}
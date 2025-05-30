use serde_json::Value;
use crate::{
    error::{MappingError, Result},
    model::rule::MappingRule,
};

pub fn resolve_constant(rule: &MappingRule) -> Result<Value> {
    rule.value
        .clone()
        .ok_or_else(|| MappingError::MissingField("value"))
}
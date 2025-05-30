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
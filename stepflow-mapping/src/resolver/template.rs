use serde_json::Value;
use tera::{Context, Tera};

use crate::{
    error::{MappingError, Result},
    model::rule::MappingRule,
};

pub fn resolve_template(rule: &MappingRule, input: &Value) -> Result<Value> {
    let tpl = rule
        .template
        .as_ref()
        .ok_or_else(|| MappingError::MissingField("template"))?;

    let mut tera = Tera::default();
    tera.add_raw_template("tpl", tpl)
        .map_err(|e| MappingError::Template(e.to_string()))?;

    // 手动绑定整个 `input` 对象到模板变量 "input"
    let mut ctx = Context::new();
    ctx.insert("input", input);

    let rendered = tera
        .render("tpl", &ctx)
        .map_err(|e| MappingError::Template(e.to_string()))?;

    Ok(Value::String(rendered))
}
use serde_json::{json, Value};

/// 构造符合 ToolInputPayload 的结构
pub fn build_tool_payload(resource: &str, input: &Value, parameters: &Value) -> Value {
    json!({
        "resource": resource,
        "input": input,
        "parameters": parameters
    })
}
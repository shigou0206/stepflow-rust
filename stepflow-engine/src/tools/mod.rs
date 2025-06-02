mod http;

use serde_json::Value;

/// 执行内置工具
pub async fn run_tool(resource: &str, input: &Value) -> Result<Value, String> {
    match resource {
        "http.get" => http::get(input).await,
        "http.post" => http::post(input).await,
        other => {
            let mut out = input.clone();
            out["_ran"] = Value::String(format!("tool::{other}"));
            Ok(out)
        }
    }
} 
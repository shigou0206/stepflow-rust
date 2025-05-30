//! Fail handler – 构造 `{ error, cause }` JSON 并原样返回。
//!
//! * 工作流在 Engine 层收到后会进入终止状态，
//!   因此这里永远 `should_continue = false`，
//!   但新接口只需返回业务 `Value`。

use serde_json::{json, Value};
use stepflow_dsl::state::fail::FailState;

/// Build the error object and return it as `Value`.
pub async fn handle_fail(
    _state_name: &str,
    state: &FailState,
    _ctx: &Value,
) -> Result<Value, String> {
    let output = json!({
        "error": state.error,
        "cause": state.cause
    });
    Ok(output)
}
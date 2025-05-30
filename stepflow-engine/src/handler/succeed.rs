use serde_json::Value;

/// Succeed handler – 把结束时的上下文 **原样返回**。
///
/// Engine 收到后会把工作流置为 Completed 状态。
pub async fn handle_succeed(
    _state_name: &str,
    ctx: &Value,
) -> Result<Value, String> {
    Ok(ctx.clone())
}
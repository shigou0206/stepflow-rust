use serde_json::Value;
use stepflow_dsl::state::pass::PassState;

/// Pass handler – 直接返回 `result` 字段（若缺省则 `{}`）。
///
/// * 是否继续由 Engine 根据 `Command::Pass { next_state, .. }` 判断，
///   这里只关心业务输出。
pub async fn handle_pass(
    _state_name: &str,
    state: &PassState,
    input: &Value,
) -> Result<Value, String> {
    let mut output = input.clone();
    if let Some(ref res) = state.result {
        if let Value::Object(map) = res {
            if let Value::Object(ref mut out_map) = output {
                for (k, v) in map {
                    out_map.insert(k.clone(), v.clone());
                }
            }
        }
    }
    Ok(output)
}
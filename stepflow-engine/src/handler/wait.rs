use std::time::Duration;
use tokio::time::sleep;
use serde_json::Value;
use stepflow_dsl::state::wait::WaitState;

/// Wait handler – inline 阻塞 `seconds`，deferred 已由外层处理定时。
///
/// 返回 **原始上下文**。
pub async fn handle_wait(
    _state_name: &str,
    state: &WaitState,
    input: &Value,
) -> Result<Value, String> {
    if let Some(secs) = state.seconds {
        sleep(Duration::from_secs(secs)).await;
    }
    // 若未来支持 Timestamp / TimerWorker，额外分支再补
    Ok(input.clone())
}
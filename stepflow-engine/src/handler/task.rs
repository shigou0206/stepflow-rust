//! Task handler – decides between **Inline** and **Deferred** execution.
//!
//! * **Inline**   → 立即调用本地工具并同步拿到结果 (`run_tool_inline`)
//! * **Deferred** → 写入任务表 + 推入队列，然后直接把 *原始输入* 返回，
//!                  由 Engine 暂停工作流等待外部 Worker 回报

use serde_json::Value;
use stepflow_dsl::state::task::TaskState;

use crate::engine::{TaskQueue, TaskStore, WorkflowMode};

// -----------------------------------------------------------------------------
// Public API
// -----------------------------------------------------------------------------
pub async fn handle_task<S, Q>(
    mode: WorkflowMode,
    _state_name: &str,
    state: &TaskState,
    input: &Value,
    store: &S,
    queue: &Q,
) -> Result<Value, String>
where
    S: TaskStore,
    Q: TaskQueue,
{
    match mode {
        // ───────────────────────────── Inline ────────────────────────────
        WorkflowMode::Inline => {
            // 真正执行业务资源 / 插件
            let output = run_tool_inline(&state.resource, input).await?;
            Ok(output) // 返回业务结果，Engine 负责 Output-Mapping
        }

        // ──────────────────────────── Deferred ───────────────────────────
        WorkflowMode::Deferred => {
            // 1️⃣ 在任务表落一条待执行记录
            store
                .insert_task(
                    "default",            // shard / run_id 这里填占位，Engine 外层会改
                    _state_name,
                    &state.resource,
                    input,
                )
                .await?;

            // 2️⃣ 推入队列，交给外部 Worker
            queue.push("default", _state_name).await?;

            // 3️⃣ 暂时返回原始输入；Engine 看到 next_state 为空会暂停
            Ok(input.clone())
        }
    }
}

// -----------------------------------------------------------------------------
// Mocked inline executor (示例)
// -----------------------------------------------------------------------------
async fn run_tool_inline(resource: &str, input: &Value) -> Result<Value, String> {
    // 演示：把原输入克隆，然后打一个 “_ran” 标记
    let mut out = input.clone();
    out["_ran"] = Value::String(format!("tool::{resource}"));
    Ok(out)
}
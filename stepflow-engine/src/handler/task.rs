//! Task handler – decides between **Inline** and **Deferred** execution.
//!
//! * **Inline**   → 立即调用本地工具并同步拿到结果 (`run_tool_inline`)
//! * **Deferred** → 写入任务表 + 推入队列，然后直接把 *原始输入* 返回，
//!                  由 Engine 暂停工作流等待外部 Worker 回报

use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use stepflow_dsl::state::task::TaskState;

use crate::engine::{TaskQueue, TaskStore, WorkflowMode};

// -----------------------------------------------------------------------------
// Public API
// -----------------------------------------------------------------------------
/// 执行 TaskState：根据 `mode` 决定是 **Inline** 立即执行工具，还是 **Deferred**
///
/// - Inline: 直接调用 `run_tool_inline`，并把工具返回的结果当成 `Value` 返回给 Engine。
/// - Deferred:
///     1. 向 `store` 写一条待办记录，`run_id` 由上层 Engine 传入，
///     2. 向 `queue` 推入一个任务，同样把 `run_id` 一并传给队列，
///     3. 返回"原始输入"给 Engine，让 Engine 暂停流程等待外部 Worker 回报。
pub async fn handle_task<S, Q>(
    mode: WorkflowMode,
    run_id: &str,         // <--- 新增：来自 Engine 的唯一 run_id
    state_name: &str,
    state: &TaskState,
    input: &Value,
    store: &S,
    queue: &Q,
    tx: &mut Transaction<'_, Sqlite>,
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
            Ok(output) // 返回业务结果，Engine 负责后续的 Output-Mapping
        }

        // ──────────────────────────── Deferred ───────────────────────────
        WorkflowMode::Deferred => {
            // 1️⃣ 在任务表落一条待执行记录；run_id 由 Engine 传入
            store
                .insert_task(
                    tx,
                    run_id,          // 由 Engine 传入的 run_id
                    state_name,
                    &state.resource,
                    input,
                )
                .await
                .map_err(|e| e.to_string())?;

            // 2️⃣ 推入队列，交给外部 Worker，注意也要把 run_id 一起传
            queue.push(tx, run_id, state_name).await
                .map_err(|e| e.to_string())?;

            // 3️⃣ 返回原始输入；Engine 看到 next_state 为空就会"挂起"流程
            Ok(input.clone())
        }
    }
}

/// Mocked inline executor (示例)
async fn run_tool_inline(resource: &str, input: &Value) -> Result<Value, String> {
    // 演示：把原输入克隆一份，然后打一个"_ran"标记
    let mut out = input.clone();
    out["_ran"] = Value::String(format!("tool::{resource}"));
    Ok(out)
}
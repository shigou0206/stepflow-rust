//! Task handler – decides between **Inline** and **Deferred** execution.
//!
//! * **Inline**   → 立即调用本地工具并同步拿到结果 (`run_tool_inline`)
//! * **Deferred** → 写入任务表 + 推入队列，然后直接把 *原始输入* 返回，
//!                  由 Engine 暂停工作流等待外部 Worker 回报

use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use stepflow_dsl::state::task::TaskState;
use crate::engine::{TaskQueue, TaskStore, WorkflowMode};
use stepflow_hook::EngineEventDispatcher;
use stepflow_storage::PersistenceManager;
use std::sync::Arc;
use crate::tools;
use async_trait::async_trait;
use tracing::debug;
use super::{StateHandler, StateExecutionContext, StateExecutionResult};


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
    run_id: &str,
    state_name: &str,
    state: &TaskState,
    input: &Value,
    store: &S,
    queue: &Q,
    tx: &mut Transaction<'_, Sqlite>,
    event_dispatcher: Arc<EngineEventDispatcher>,
    persistence: Arc<dyn PersistenceManager>,
) -> Result<Value, String>
where
    S: TaskStore,
    Q: TaskQueue,
{
    let _ = tx;
    let ctx = StateExecutionContext::new(
        run_id,
        state_name,
        mode,
        &event_dispatcher,
        &persistence,
    );

    let handler = TaskHandler::new(store, queue, state);
    let result = handler.execute(&ctx, input).await?;
    
    Ok(result.output)
}

pub struct TaskHandler<'a, S: TaskStore, Q: TaskQueue> {
    store: &'a S,
    queue: &'a Q,
    state: &'a TaskState,
}

impl<'a, S: TaskStore, Q: TaskQueue> TaskHandler<'a, S, Q> {
    pub fn new(
        store: &'a S,
        queue: &'a Q,
        state: &'a TaskState,
    ) -> Self {
        Self {
            store,
            queue,
            state,
        }
    }

    async fn handle_inline(&self, input: &Value) -> Result<Value, String> {
        debug!("Executing task inline with resource: {}", self.state.resource);
        tools::run_tool(&self.state.resource, input).await
    }

    async fn handle_deferred(
        &self,
        ctx: &StateExecutionContext<'_>,
        input: &Value,
    ) -> Result<Value, String> {
        debug!("Creating deferred task for resource: {}", self.state.resource);

        // 获取事务作为参数
        let mut tx = sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap()
            .begin()
            .await
            .map_err(|e| e.to_string())?;

        // 1. 在任务表落一条待执行记录
        self.store
            .insert_task(
                &mut tx,
                ctx.run_id,
                ctx.state_name,
                &self.state.resource,
                input,
            )
            .await
            .map_err(|e| e.to_string())?;

        // 2. 推入队列，交给外部 Worker
        self.queue
            .push(&mut tx, ctx.run_id, ctx.state_name)
            .await
            .map_err(|e| e.to_string())?;

        // 提交事务
        tx.commit()
            .await
            .map_err(|e| e.to_string())?;

        // 3. 返回原始输入，Engine 会"挂起"流程
        Ok(input.clone())
    }
}

#[async_trait(?Send)]
impl<'a, S: TaskStore, Q: TaskQueue> StateHandler for TaskHandler<'a, S, Q> {
    async fn handle(
        &self,
        ctx: &StateExecutionContext<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let output = match ctx.mode {
            WorkflowMode::Inline => self.handle_inline(input).await?,
            WorkflowMode::Deferred => self.handle_deferred(ctx, input).await?,
        };

        Ok(StateExecutionResult {
            output,
            next_state: self.state.base.next.clone(),
            should_continue: true,
        })
    }

    fn state_type(&self) -> &'static str {
        "task"
    }
}

//! Task handler – decides between **Inline** and **Deferred** execution.
//!
//! * **Inline**   → 立即调用本地工具并同步拿到结果 (`run_tool_inline`)
//! * **Deferred** → 写入任务表 + 推入队列，然后直接把 *原始输入* 返回，
//!                  由 Engine 暂停工作流等待外部 Worker 回报

use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use stepflow_dsl::state::task::TaskState;
use crate::engine::{TaskStore, WorkflowMode};
use crate::match_service::{MatchService, Task as MatchServiceTask};
use stepflow_hook::EngineEventDispatcher;
use stepflow_storage::persistence_manager::PersistenceManager;
use std::sync::Arc;
use crate::tools;
use async_trait::async_trait;
use tracing::{debug, warn};
use super::{StateHandler, StateExecutionContext, StateExecutionResult};


// -----------------------------------------------------------------------------
// Public API
// -----------------------------------------------------------------------------
/// Executes TaskState: decides between Inline execution or Deferred dispatch via MatchService.
pub async fn handle_task(
    mode: WorkflowMode,
    run_id: &str,
    state_name: &str,
    state: &TaskState,
    input: &Value,
    _store: &impl TaskStore, // Keep a generic TaskStore reference if other parts of handle_task use it, or remove if not.
    match_service: Arc<dyn MatchService>,
    _tx: &mut Transaction<'_, Sqlite>,
    event_dispatcher: Arc<EngineEventDispatcher>,
    persistence: Arc<dyn PersistenceManager>,
) -> Result<Value, String>
{
    let ctx = StateExecutionContext::new(
        run_id,
        state_name,
        mode,
        &event_dispatcher,
        &persistence,
    );

    let handler = TaskHandler::new(match_service, state);
    let result = handler.execute(&ctx, input).await?;
    
    Ok(result.output)
}

pub struct TaskHandler<'a> { // Removed generic <S: TaskStore>
    match_service: Arc<dyn MatchService>,
    state: &'a TaskState,
    // _marker: std::marker::PhantomData<S>, // Removed PhantomData
}

impl<'a> TaskHandler<'a> { // Removed generic <S: TaskStore>
    pub fn new(
        match_service: Arc<dyn MatchService>,
        state: &'a TaskState,
    ) -> Self {
        Self {
            match_service,
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
        debug!("Creating deferred task for resource: {} via MatchService", self.state.resource);

        // TODO: Define how priority and timeout_seconds are sourced from TaskState's DSL definition.
        //       Currently attempting to parse from execution_config, defaulting to None.
        let mut priority = None;
        let mut timeout_seconds = None;

        if let Some(config) = &self.state.execution_config {
            if let Some(p_val) = config.get("priority") {
                if let Some(p) = p_val.as_u64().and_then(|v| u8::try_from(v).ok()) {
                    priority = Some(p);
                } else {
                    warn!("Could not parse 'priority' from execution_config for task: {}", ctx.state_name);
                }
            }
            if let Some(t_val) = config.get("timeout_seconds") {
                if let Some(t) = t_val.as_i64() {
                    timeout_seconds = Some(t);
                } else {
                    warn!("Could not parse 'timeout_seconds' from execution_config for task: {}", ctx.state_name);
                }
            }
        }

        let service_task = MatchServiceTask {
            run_id: ctx.run_id.to_string(),
            state_name: ctx.state_name.to_string(),
            input: Some(input.clone()),
            task_type: self.state.resource.clone(),
            task_token: None,
            priority, // Sourced from execution_config or None
            attempt: Some(0),
            max_attempts: None, // TODO: Source from TaskState if defined (e.g., execution_config or direct field)
            timeout_seconds, // Sourced from execution_config or None
            scheduled_at: None,
        };
        
        self.match_service
            .enqueue_task(&self.state.resource, service_task)
            .await
            .map_err(|e| format!("Failed to enqueue task via MatchService: {}", e))?;

        Ok(input.clone())
    }
}

#[async_trait]
impl<'a> StateHandler for TaskHandler<'a> {
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

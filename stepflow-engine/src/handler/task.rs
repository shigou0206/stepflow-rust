//! Task handler – decides between **Inline** and **Deferred** execution.
//!
//! * **Inline**   → 立即调用本地工具并同步拿到结果 (`run_tool_inline`)
//! * **Deferred** → 写入任务表 + 推入队列，然后直接把 *原始输入* 返回，
//!                  由 Engine 暂停工作流等待外部 Worker 回报

use serde_json::Value;
use stepflow_dsl::state::task::TaskState;
use stepflow_hook::EngineEventDispatcher;
use stepflow_storage::persistence_manager::PersistenceManager;
use crate::engine::{WorkflowMode};
use stepflow_match::service::{MatchService, Task as MatchServiceTask};
use stepflow_match::queue::TaskStore;
use std::sync::Arc;
use async_trait::async_trait;
use tracing::{debug, warn};
use super::{StateHandler, StateExecutionContext, StateExecutionResult};


// -----------------------------------------------------------------------------
// Public API
// -----------------------------------------------------------------------------
/// Executes TaskState: decides between Inline execution or Deferred dispatch via MatchService.
pub async fn handle_task<S: TaskStore>(
    mode: WorkflowMode,
    run_id: &str,
    state_name: &str,
    state: &TaskState,
    input: &Value,
    store: &S,
    match_service: Arc<dyn MatchService>,
    persistence: &Arc<dyn PersistenceManager>,
    _event_dispatcher: Arc<EngineEventDispatcher>,
    persistence_for_handlers: Arc<dyn PersistenceManager>,
) -> Result<Value, String> {
    // 创建任务记录
    store.insert_task(
        persistence,
        run_id,
        state_name,
        &state.resource,
        input,
    ).await?;

    // 如果是 inline 模式，等待任务完成
    if mode == WorkflowMode::Inline {
        // 等待任务完成
        match_service.wait_for_completion(
            run_id,
            state_name,
            input,
            persistence_for_handlers,
        ).await
    } else {
        // deferred 模式，直接返回输入
        Ok(input.clone())
    }
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
        crate::tools::run_tool(&self.state.resource, input).await
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
            priority,
            attempt: Some(0),
            max_attempts: Some(3), // 默认最多重试3次
            timeout_seconds,
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

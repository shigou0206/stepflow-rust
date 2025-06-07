//! Task handler – decides between **Inline** and **Deferred** execution.
//!
//! * **Inline**   → 立即调用本地工具并同步拿到结果 (`run_tool_inline`)
//! * **Deferred** → 写入任务表 + 推入队列，然后直接把 *原始输入* 返回，
//!                  由 Engine 暂停工作流等待外部 Worker 回报

use serde_json::Value;
use stepflow_dsl::state::task::TaskState;
use stepflow_storage::persistence_manager::PersistenceManager;
use crate::engine::WorkflowMode;
use stepflow_match::service::{MatchService, Task as MatchServiceTask};
use std::sync::Arc;
use async_trait::async_trait;
use tracing::{debug, warn};
use super::{StateHandler, StateExecutionContext, StateExecutionResult};

/// Executes TaskState: decides between Inline execution or Deferred dispatch via MatchService.
pub async fn handle_task(
    mode: WorkflowMode,
    run_id: &str,
    state_name: &str,
    state: &TaskState,
    input: &Value,
    match_service: Arc<dyn MatchService>,
    persistence_for_handlers: Arc<dyn PersistenceManager>,
) -> Result<Value, String> {
    // 构造任务
    let service_task = MatchServiceTask {
        run_id: run_id.to_string(),
        state_name: state_name.to_string(),
        input: Some(input.clone()),
        task_type: state.resource.clone(),
        task_token: None,
        priority: None,
        attempt: Some(0),
        max_attempts: Some(3),
        timeout_seconds: None,
        scheduled_at: None,
    };

    match_service
        .enqueue_task(&state.resource, service_task)
        .await
        .map_err(|e| format!("enqueue failed: {e}"))?;

    if mode == WorkflowMode::Inline {
        match_service
            .wait_for_completion(run_id, state_name, input, persistence_for_handlers)
            .await
    } else {
        Ok(input.clone())
    }
}

pub struct TaskHandler<'a> {
    match_service: Arc<dyn MatchService>,
    state: &'a TaskState,
}

impl<'a> TaskHandler<'a> {
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
            max_attempts: Some(3),
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
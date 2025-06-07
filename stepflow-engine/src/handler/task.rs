use chrono::Utc;
use serde_json::Value;
use stepflow_dsl::state::task::TaskState;
use stepflow_storage::persistence_manager::PersistenceManager;
use crate::engine::WorkflowMode;
use stepflow_match::service::MatchService;
use stepflow_dto::dto::queue_task::QueueTaskDto;

use std::sync::Arc;
use async_trait::async_trait;
use tracing::{debug, warn};

use super::{StateHandler, StateExecutionContext, StateExecutionResult};

/// 从 TaskState 的 execution_config 中提取 priority 和 timeout
fn extract_priority_and_timeout(
    state: &TaskState,
    run_id: &str,
    state_name: &str,
) -> (Option<u8>, Option<i64>) {
    let mut priority = None;
    let mut timeout_seconds = None;

    if let Some(config) = &state.execution_config {
        if let Some(p_val) = config.get("priority") {
            if let Some(p) = p_val.as_u64().and_then(|v| u8::try_from(v).ok()) {
                priority = Some(p);
            } else {
                warn!("Invalid priority in config for {}.{}", run_id, state_name);
            }
        }

        if let Some(t_val) = config.get("timeout_seconds") {
            if let Some(t) = t_val.as_i64() {
                timeout_seconds = Some(t);
            } else {
                warn!("Invalid timeout_seconds in config for {}.{}", run_id, state_name);
            }
        }
    }

    (priority, timeout_seconds)
}

/// 构造完整的 QueueTaskDto（不含 task_id）
fn build_queue_task(
    run_id: &str,
    state_name: &str,
    state: &TaskState,
    input: &Value,
) -> QueueTaskDto {
    let (priority, timeout_seconds) = extract_priority_and_timeout(state, run_id, state_name);

    QueueTaskDto {
        task_id: "".to_string(), // 留空，由持久化层生成
        run_id: run_id.to_string(),
        state_name: state_name.to_string(),
        resource: state.resource.clone(),
        task_payload: Some(input.clone()),
        status: "pending".to_string(),
        attempts: 0,
        max_attempts: 3,
        priority,
        timeout_seconds,
        error_message: None,
        last_error_at: None,
        next_retry_at: None,
        queued_at: Utc::now(),
        processing_at: None,
        completed_at: None,
        failed_at: None,
    }
}

/// 通用 Task 状态处理函数
pub async fn handle_task(
    mode: WorkflowMode,
    run_id: &str,
    state_name: &str,
    state: &TaskState,
    input: &Value,
    match_service: Arc<dyn MatchService>,
    persistence_for_handlers: Arc<dyn PersistenceManager>,
) -> Result<Value, String> {
    let service_task = build_queue_task(run_id, state_name, state, input);

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

/// TaskHandler 结构体，用于状态分发
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
        debug!(
            "Creating deferred task for resource: {} via MatchService",
            self.state.resource
        );

        let task = build_queue_task(ctx.run_id, ctx.state_name, self.state, input);

        self.match_service
            .enqueue_task(&self.state.resource, task)
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
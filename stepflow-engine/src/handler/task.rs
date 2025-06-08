use chrono::Utc;
use serde_json::Value;
use stepflow_dsl::state::task::TaskState;
use stepflow_match::queue::DynPM;
use crate::engine::WorkflowMode;
use stepflow_match::service::MatchService;
use stepflow_dto::dto::queue_task::QueueTaskDto;
use stepflow_dto::dto::tool::ToolInputPayload;
use stepflow_tool::common::context::ToolContext;
use stepflow_tool::registry::globals::GLOBAL_TOOL_REGISTRY;

use std::sync::Arc;
use async_trait::async_trait;
use tracing::{debug, warn};

use super::{StateHandler, StateExecutionContext, StateExecutionResult};

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
                warn!("Invalid priority in config for {}.{}, using None", run_id, state_name);
            }
        }

        if let Some(t_val) = config.get("timeout_seconds") {
            if let Some(t) = t_val.as_i64() {
                timeout_seconds = Some(t);
            } else {
                warn!("Invalid timeout_seconds in config for {}.{}, using None", run_id, state_name);
            }
        }
    }

    (priority, timeout_seconds)
}

fn build_queue_task(
    run_id: &str,
    state_name: &str,
    state: &TaskState,
    input: &Value,
) -> QueueTaskDto {
    let (priority, timeout_seconds) = extract_priority_and_timeout(state, run_id, state_name);

    QueueTaskDto {
        task_id: "".to_string(),
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

pub async fn handle_task(
    mode: WorkflowMode,
    run_id: &str,
    state_name: &str,
    state: &TaskState,
    input: &Value,
    match_service: Arc<dyn MatchService>,
    persistence_for_handlers: &DynPM,
) -> Result<Value, String> {
    let payload = ToolInputPayload::build(&state.resource, input)?;
    let input_value = serde_json::to_value(payload).map_err(|e| e.to_string())?;

    use tracing::debug;

    debug!("handle_task - mapped input: {:?}", input);

    let service_task = build_queue_task(run_id, state_name, state, &input_value);

    match_service
        .enqueue_task(&state.resource, service_task)
        .await
        .map_err(|e| format!("enqueue failed: {e}"))?;

    if mode == WorkflowMode::Inline {
        match_service
            .wait_for_completion(run_id, state_name, &input_value, persistence_for_handlers)
            .await
    } else {
        Ok(input_value)
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

        let payload = ToolInputPayload::build(&self.state.resource, input)
            .map_err(|e| format!("Failed to build tool payload: {}", e))?;

        let tool = {
            let registry = GLOBAL_TOOL_REGISTRY
                .lock()
                .map_err(|e| format!("Failed to lock tool registry: {}", e))?;

            registry
                .get(&payload.resource)
                .ok_or_else(|| format!("Tool not found: {}", payload.resource))?
                .clone()
        };

        let context = ToolContext::default();

        tool.validate_input(&payload.parameters, &context)
            .map_err(|e| format!("Tool input validation failed: {}", e))?;

        let result = tool
            .execute(payload.parameters.clone(), context)
            .await
            .map_err(|e| format!("Tool execution failed: {}", e))?;

        Ok(result.output)
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

        let payload = ToolInputPayload::build(&self.state.resource, input)?;
        let input_value = serde_json::to_value(payload).map_err(|e| e.to_string())?;

        let task = build_queue_task(ctx.run_id, ctx.state_name, self.state, &input_value);

        self.match_service
            .enqueue_task(&self.state.resource, task)
            .await
            .map_err(|e| format!("Failed to enqueue task via MatchService: {}", e))?;

        Ok(input_value)
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
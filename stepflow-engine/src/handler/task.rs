use chrono::Utc;
use serde_json::Value;
use stepflow_dsl::state::{task::TaskState, State};
use stepflow_dto::dto::queue_task::QueueTaskDto;
use stepflow_match::service::MatchService;

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, warn};

use super::{StateExecutionResult, StateExecutionScope, StateHandler};

pub struct TaskHandler {
    match_service: Arc<dyn MatchService>,
}

impl TaskHandler {
    pub fn new(match_service: Arc<dyn MatchService>) -> Self {
        Self { match_service }
    }

    async fn handle_deferred(
        &self,
        scope: &StateExecutionScope<'_>,
        state: &TaskState,
        input: &Value,
    ) -> Result<(Value, Value), String> {
        debug!(
            "Creating deferred task for resource: {} via MatchService",
            state.resource
        );

        let task = build_queue_task(scope.run_id, scope.state_name, state, input);

        self.match_service
            .enqueue_task(&state.resource, task.clone())
            .await
            .map_err(|e| format!("Failed to enqueue task via MatchService: {}", e))?;

        let metadata = serde_json::to_value(&task)
            .map_err(|e| format!("Failed to serialize task metadata: {}", e))?;

        Ok((input.clone(), metadata))
    }
}

#[async_trait]
impl StateHandler for TaskHandler {
    async fn handle(
        &self,
        scope: &StateExecutionScope<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Task(ref s) => s,
            _ => return Err("Invalid state type for TaskHandler".into()),
        };

        let (output, metadata) = {
            let (out, meta) = self.handle_deferred(scope, state, input).await?;
            (out, Some(meta))
        };

        Ok(StateExecutionResult {
            output,
            next_state: state.base.next.clone(),
            should_continue: true,
            metadata,
        })
    }

    fn state_type(&self) -> &'static str {
        "task"
    }

    async fn on_subflow_finished(
        &self,
        _scope: &StateExecutionScope<'_>,
        _parent_context: &Value,
        _child_run_id: &str,
        _result: &Value,
    ) -> Result<StateExecutionResult, String> {
        Err("on_subflow_finished not supported by this state".into())
    }
}

// 保留任务构建辅助函数
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
                warn!(
                    "Invalid priority in config for {}.{}, using None",
                    run_id, state_name
                );
            }
        }

        if let Some(t_val) = config.get("timeout_seconds") {
            if let Some(t) = t_val.as_i64() {
                timeout_seconds = Some(t);
            } else {
                warn!(
                    "Invalid timeout_seconds in config for {}.{}, using None",
                    run_id, state_name
                );
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

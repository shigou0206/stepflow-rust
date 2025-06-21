use super::{StateExecutionResult, StateExecutionScope, StateHandler};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use stepflow_dsl::state::State;
use stepflow_match::service::SubflowMatchService;
use stepflow_storage::entities::workflow_execution::StoredWorkflowExecution;
use std::sync::Arc;
use tracing::{debug, error};

pub struct ParallelHandler {
    pub subflow_match: Arc<dyn SubflowMatchService>,
}

impl ParallelHandler {
    pub fn new(subflow_match: Arc<dyn SubflowMatchService>) -> Self {
        Self { subflow_match }
    }
}

#[async_trait]
impl StateHandler for ParallelHandler {
    async fn handle(
        &self,
        scope: &StateExecutionScope<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Parallel(ref s) => s,
            _ => return Err("Invalid state type for ParallelHandler".into()),
        };

        let max_concurrency = state.max_concurrency.unwrap_or(state.branches.len() as u32);
        debug!(max_concurrency, total = state.branches.len(), "🧵 Creating parallel subflows");

        for (index, branch) in state.branches.iter().enumerate() {
            let status = if index < max_concurrency as usize {
                "READY"
            } else {
                "WAITING"
            };

            let child_run_id = format!("{}:{}:{}", scope.run_id, scope.state_name, index);
            let dsl_def = serde_json::to_value(branch).map_err(|e| e.to_string())?;

            let subflow = StoredWorkflowExecution {
                run_id: child_run_id.clone(),
                workflow_id: Some(child_run_id.clone()),
                shard_id: 0,
                template_id: None,
                mode: "DEFERRED".into(), // ✅ 固定为 DEFERRED
                current_state_name: Some(branch.start_at.clone()),
                status: status.into(),
                workflow_type: "parallel_subflow".into(),
                input: Some(input.clone()),
                input_version: 1,
                result: None,
                result_version: 1,
                start_time: Utc::now().naive_utc(),
                close_time: None,
                current_event_id: 0,
                memo: None,
                search_attrs: None,
                context_snapshot: Some(input.clone()),
                version: 1,
                parent_run_id: Some(scope.run_id.to_string()),
                parent_state_name: Some(scope.state_name.to_string()),
                dsl_definition: Some(dsl_def.clone()),
            };

            scope
                .persistence
                .create_execution(&subflow)
                .await
                .map_err(|e| {
                    error!(%child_run_id, ?e, "❌ Failed to insert parallel subflow");
                    format!("Failed to create subflow: {e}")
                })?;

            if status == "READY" {
                self.subflow_match
                    .notify_subflow_ready(
                        child_run_id,
                        scope.run_id.to_string(),
                        scope.state_name.to_string(),
                        dsl_def.clone(),
                        input.clone(),
                    )
                    .await
                    .map_err(|e| {
                        error!(?e, "❌ notify_subflow_ready failed");
                        e
                    })?;
            }
        }

        Ok(StateExecutionResult {
            output: input.clone(),
            next_state: None,
            should_continue: true,
            metadata: Some(json!({ "subflows": state.branches.len() })),
        })
    }

    fn state_type(&self) -> &'static str {
        "parallel"
    }

    async fn on_subflow_finished(
        &self,
        scope: &StateExecutionScope<'_>,
        parent_context: &Value,
        _child_run_id: &str,
        _result: &Value,
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Parallel(ref s) => s,
            _ => return Err("Invalid state type for ParallelHandler".into()),
        };

        let subflows = scope
            .persistence
            .find_subflows_by_parent(scope.run_id, scope.state_name)
            .await
            .map_err(|e| e.to_string())?;

        let all_done = subflows.iter().all(|s| s.status == "COMPLETED");

        if all_done {
            let results: Vec<_> = subflows
                .iter()
                .map(|s| s.result.clone().unwrap_or(Value::Null))
                .collect();

            let mut output = parent_context.clone();
            output["parallelResult"] = Value::Array(results);

            Ok(StateExecutionResult {
                output,
                next_state: scope.next().cloned(),
                should_continue: true,
                metadata: None,
            })
        } else {
            if let Some(waiting) = subflows.iter().find(|s| s.status == "WAITING") {
                self.subflow_match
                    .notify_subflow_ready(
                        waiting.run_id.clone(),
                        scope.run_id.to_string(),
                        scope.state_name.to_string(),
                        json!(state.branches[index_of(&subflows, &waiting.run_id)]),
                        waiting.input.clone().unwrap_or_default(),
                    )
                    .await?;
            }

            Ok(StateExecutionResult {
                output: parent_context.clone(),
                next_state: None,
                should_continue: true,
                metadata: None,
            })
        }
    }
}

// 工具函数：查找 waiting 子流程的索引
fn index_of(subflows: &[StoredWorkflowExecution], run_id: &str) -> usize {
    subflows
        .iter()
        .position(|s| s.run_id == run_id)
        .unwrap_or(0) // fallback：应该不发生
}
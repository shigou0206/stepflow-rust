use super::{StateExecutionResult, StateExecutionScope, StateHandler};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::sync::Arc;
use stepflow_dsl::state::State;
use stepflow_match::service::SubflowMatchService;
use crate::mapping::MappingPipeline;
use stepflow_storage::entities::workflow_execution::StoredWorkflowExecution;
use tracing::{debug, error, warn};

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
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Parallel(ref s) => s,
            _ => return Err("Invalid state type for ParallelHandler".into()),
        };

        debug!(run_id = scope.run_id, state = scope.state_name, "🔀 Entered ParallelHandler");

        let pipeline = MappingPipeline {
            input_mapping: state.base.input_mapping.as_ref(),
            output_mapping: None,
        };

        for (index, branch) in state.branches.iter().enumerate() {
            let child_run_id = format!("{}:{}:{}", scope.run_id, scope.state_name, index);
            let exec_input = pipeline.apply_input(scope.context)?;
            let context_snapshot = exec_input.clone();

            let subflow = StoredWorkflowExecution {
                run_id: child_run_id.clone(),
                workflow_id: Some(child_run_id.clone()),
                shard_id: 0,
                template_id: None,
                mode: "DEFERRED".to_string(),
                current_state_name: Some(branch.start_at.clone()),
                status: "READY".to_string(),
                workflow_type: "parallel_subflow".to_string(),
                input: Some(exec_input.clone()),
                input_version: 1,
                result: None,
                result_version: 1,
                start_time: Utc::now().naive_utc(),
                close_time: None,
                current_event_id: 0,
                memo: None,
                search_attrs: None,
                context_snapshot: Some(context_snapshot),
                version: 1,
                parent_run_id: Some(scope.run_id.to_string()),
                parent_state_name: Some(scope.state_name.to_string()),
                dsl_definition: Some(json!(branch)),
            };

            scope.persistence.create_execution(&subflow).await.map_err(|e| {
                error!(%child_run_id, ?e, "❌ Failed to insert subflow");
                format!("Failed to create subflow: {e}")
            })?;

            self.subflow_match
                .notify_subflow_ready(
                    child_run_id,
                    scope.run_id.to_string(),
                    scope.state_name.to_string(),
                    json!(branch),
                    exec_input,
                )
                .await
                .map_err(|e| {
                    error!(%scope.run_id, "❌ notify_subflow_ready failed: {}", e);
                    e
                })?;
        }

        Ok(StateExecutionResult {
            output: scope.context.clone(),
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
        debug!(run_id = scope.run_id, state = scope.state_name, "🧩 on_subflow_finished (parallel)");

        let state = match scope.state_def {
            State::Parallel(ref s) => s,
            _ => return Err("Invalid state type for ParallelHandler".into()),
        };

        let subflows = scope
            .persistence
            .find_subflows_by_parent(scope.run_id, scope.state_name)
            .await
            .map_err(|e| {
                error!(run_id = scope.run_id, "❌ Failed to query subflows: {}", e);
                e.to_string()
            })?;

        let all_done = subflows.iter().all(|s| s.status == "COMPLETED");
        debug!(total = subflows.len(), all_done, "🔎 Parallel subflow completion check");

        if all_done {
            Ok(StateExecutionResult {
                output: parent_context.clone(),
                next_state: scope.next().cloned(),
                should_continue: true,
                metadata: None,
            })
        } else {
            Ok(StateExecutionResult {
                output: parent_context.clone(),
                next_state: None,
                should_continue: true,
                metadata: None,
            })
        }
    }
}
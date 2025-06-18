use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use stepflow_dsl::state::State;
use stepflow_storage::entities::workflow_execution::StoredWorkflowExecution;

use super::{StateExecutionScope, StateExecutionResult, StateHandler};

pub struct ParallelHandler;

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

        let mut subflow_ids = Vec::new();

        for (index, branch) in state.branches.iter().enumerate() {
            let child_run_id = format!("{}:{}:{}", scope.run_id, scope.state_name, index);
            let dsl_def = serde_json::to_value(branch).map_err(|e| e.to_string())?;

            let subflow = StoredWorkflowExecution {
                run_id: child_run_id.clone(),
                workflow_id: None,
                shard_id: 0,
                template_id: None,
                mode: format!("{:?}", scope.mode),
                current_state_name: Some(branch.start_at.clone()),
                status: "CREATED".into(),
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
                context_snapshot: None,
                version: 1,
                parent_run_id: Some(scope.run_id.to_string()),
                parent_state_name: Some(scope.state_name.to_string()),
                dsl_definition: Some(dsl_def),
            };

            scope
                .persistence
                .create_execution(&subflow)
                .await
                .map_err(|e| format!("Failed to create subflow: {e}"))?;

            subflow_ids.push(child_run_id);
        }

        Ok(StateExecutionResult {
            output: input.clone(),
            next_state: None,
            should_continue: false,
            metadata: Some(json!({ "subflows": subflow_ids })),
        })
    }

    fn state_type(&self) -> &'static str {
        "parallel"
    }
}
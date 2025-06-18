use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use jsonpath_lib;
use stepflow_dsl::state::State;
use stepflow_storage::entities::workflow_execution::StoredWorkflowExecution;
use super::{StateHandler, StateExecutionScope, StateExecutionResult};

pub struct MapHandler;

#[async_trait]
impl StateHandler for MapHandler {
    async fn handle(
        &self,
        scope: &StateExecutionScope<'_>,
        input: &Value,
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Map(ref s) => s,
            _ => return Err("Invalid state type for MapHandler".into()),
        };

        let items = jsonpath_lib::select(input, &state.items_path)
            .map_err(|e| format!("Invalid itemsPath: {e}"))?;

        let items = match items.first() {
            Some(Value::Array(arr)) => arr.clone(),
            _ => return Err("itemsPath must resolve to an array".into()),
        };

        let mut subflow_ids = Vec::new();

        for (index, item) in items.iter().enumerate() {
            let child_run_id = format!("{}:{}:{}", scope.run_id, scope.state_name, index);
            let child_context = json!({"item": item});

            let dsl_def = serde_json::to_string(&state.iterator).map_err(|e| e.to_string())?;

            let subflow = StoredWorkflowExecution {
                run_id: child_run_id.clone(),
                workflow_id: None,
                shard_id: 0,
                template_id: None,
                mode: format!("{:?}", scope.mode),
                current_state_name: Some(state.iterator.start_at.clone()),
                status: "CREATED".to_string(),
                workflow_type: "map_subflow".to_string(),
                input: Some(child_context),
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
                dsl_definition: Some(serde_json::from_str(&dsl_def).map_err(|e| e.to_string())?),
            };

            scope.persistence.create_execution(&subflow)
                .await.map_err(|e| format!("Failed to create subflow: {e}"))?;

            subflow_ids.push(child_run_id);
        }

        Ok(StateExecutionResult {
            output: input.clone(),
            next_state: None,
            should_continue: false,
            metadata: Some(json!({"subflows": subflow_ids})),
        })
    }

    fn state_type(&self) -> &'static str {
        "map"
    }
}

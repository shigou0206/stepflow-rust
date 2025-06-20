use super::{StateExecutionResult, StateExecutionScope, StateHandler};
use async_trait::async_trait;
use chrono::Utc;
use jsonpath_lib;
use serde_json::{json, Value};
use std::sync::Arc;
use stepflow_dsl::state::State;
use stepflow_match::service::SubflowMatchService;
use stepflow_storage::entities::workflow_execution::StoredWorkflowExecution;
use tracing::{debug, error, warn};

pub struct MapHandler {
    pub subflow_match: Arc<dyn SubflowMatchService>,
}

impl MapHandler {
    pub fn new(subflow_match: Arc<dyn SubflowMatchService>) -> Self {
        Self { subflow_match }
    }
}

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

        debug!(run_id = scope.run_id, state = scope.state_name, "🧭 Entered MapHandler");

        let matched = jsonpath_lib::select(input, &state.items_path)
            .map_err(|e| format!("Invalid itemsPath: {e}"))?;

        let items: Vec<Value> = matched.into_iter().cloned().collect();
        debug!(matched_len = items.len(), path = %state.items_path, "🔍 JsonPath matched items");

        if items.is_empty() {
            warn!(run_id = scope.run_id, "⚠️ itemsPath yielded no items");
            return Err("itemsPath did not yield any array items".into());
        }

        let max_concurrency = state.max_concurrency.unwrap_or(items.len() as u32);
        debug!(max_concurrency, total_items = items.len(), "🧵 Creating subflows");

        for (index, item) in items.iter().enumerate() {
            let child_run_id = format!("{}:{}:{}", scope.run_id, scope.state_name, index);
            let child_context = json!({ "item": item });
            let status = if index < max_concurrency as usize { "READY" } else { "WAITING" };

            debug!(%child_run_id, %status, "🚧 Inserting subflow");

            let subflow = StoredWorkflowExecution {
                run_id: child_run_id.clone(),
                workflow_id: Some(child_run_id.clone()),
                shard_id: 0,
                template_id: None,
                mode: "DEFERRED".to_string(), // 强制写入 "DEFERRED"
                current_state_name: Some(state.iterator.start_at.clone()),
                status: status.to_string(),
                workflow_type: "map_subflow".to_string(),
                input: Some(child_context.clone()),
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
                dsl_definition: Some(json!(state.iterator)),
            };

            scope.persistence
                .create_execution(&subflow)
                .await
                .map_err(|e| {
                    error!(%child_run_id, ?e, "❌ Failed to insert subflow");
                    format!("Failed to create subflow: {e}")
                })?;

            if status == "READY" {
                debug!(%child_run_id, "📬 Emitting subflow READY event");
                self.subflow_match
                    .notify_subflow_ready(
                        child_run_id,
                        scope.run_id.to_string(),
                        scope.state_name.to_string(),
                        json!(state.iterator),
                        child_context,
                    )
                    .await
                    .map_err(|e| {
                        error!(%scope.run_id, "❌ notify_subflow_ready failed: {}", e);
                        e
                    })?;
            }
        }

        Ok(StateExecutionResult {
            output: input.clone(),
            next_state: None,
            should_continue: true,
            metadata: Some(json!({ "subflows": items.len() })),
        })
    }

    fn state_type(&self) -> &'static str {
        "map"
    }

    async fn on_subflow_finished(
        &self,
        scope: &StateExecutionScope<'_>,
        parent_context: &Value,
        _child_run_id: &str,
        _result: &Value,
    ) -> Result<StateExecutionResult, String> {
        debug!(run_id = scope.run_id, state = scope.state_name, "🧩 on_subflow_finished triggered");
    
        // ✅ 从 scope 中提取 MapState
        let state = match scope.state_def {
            State::Map(ref s) => s,
            _ => return Err("Invalid state type for MapHandler".into()),
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
        debug!(total = subflows.len(), all_done, "🔎 Subflow completion check");
    
        if all_done {
            let results: Vec<_> = subflows
                .iter()
                .map(|s| s.result.clone().unwrap_or(Value::Null))
                .collect();
    
            let mut output = parent_context.clone();
            output["mapResult"] = Value::Array(results);
    
            debug!(run_id = scope.run_id, state = scope.state_name, "✅ All subflows done, continuing");
    
            Ok(StateExecutionResult {
                output,
                next_state: scope.next().cloned(),
                should_continue: true,
                metadata: None,
            })
        } else {
            if let Some(waiting) = subflows.iter().find(|s| s.status == "WAITING") {
                debug!(waiting_id = %waiting.run_id, "🔁 Resuming waiting subflow");
    
                self.subflow_match
                    .notify_subflow_ready(
                        waiting.run_id.clone(),
                        scope.run_id.to_string(),
                        scope.state_name.to_string(),
                        json!(state.iterator), // ✅ 现在可以访问 state
                        waiting.input.clone().unwrap_or_default(),
                    )
                    .await
                    .map_err(|e| {
                        error!(%scope.run_id, "❌ notify_subflow_ready (waiting) failed: {}", e);
                        e
                    })?;
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
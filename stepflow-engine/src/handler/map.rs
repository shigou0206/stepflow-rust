use super::{StateExecutionResult, StateExecutionScope, StateHandler};
use crate::mapping::MappingPipeline;
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
    ) -> Result<StateExecutionResult, String> {
        let state = match scope.state_def {
            State::Map(ref s) => s,
            _ => return Err("Invalid state type for MapHandler".into()),
        };

        let item_key = state.item_context_key.as_str();
        let parent_context = scope.context;
        debug!(?parent_context, "📦 Current context");
        debug!(
            run_id = scope.run_id,
            state = scope.state_name,
            "🧭 Entered MapHandler"
        );

        let matched = jsonpath_lib::select(parent_context, &state.items_path)
            .map_err(|e| format!("Invalid itemsPath: {e}"))?;

        let items: Vec<Value> = if matched.len() == 1 && matched[0].is_array() {
            matched[0]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .cloned()
                .collect()
        } else {
            matched.iter().map(|v| (*v).clone()).collect()
        };

        debug!(matched_len = items.len(), path = %state.items_path, "🔍 JsonPath matched items");
        if items.is_empty() {
            warn!(run_id = scope.run_id, "⚠️ itemsPath yielded no items");
            return Err("itemsPath did not yield any array items".into());
        }

        let max_concurrency = state.max_concurrency.unwrap_or(items.len() as u32);
        let pipeline = MappingPipeline {
            input_mapping: state.base.input_mapping.as_ref(),
            output_mapping: None,
        };

        for (index, item) in items.iter().enumerate() {
            debug!(index, ?item, "📦 Processing item");

            let child_run_id = format!("{}:{}:{}", scope.run_id, scope.state_name, index);
            let mapped_input = pipeline.apply_input_for_map_item(parent_context, item_key, item)?;

            let status = if index < max_concurrency as usize {
                "READY"
            } else {
                "WAITING"
            };

            let subflow = StoredWorkflowExecution {
                run_id: child_run_id.clone(),
                workflow_id: Some(child_run_id.clone()),
                shard_id: 0,
                template_id: None,
                mode: "DEFERRED".to_string(),
                current_state_name: Some(state.iterator.start_at.clone()),
                status: status.to_string(),
                workflow_type: "map_subflow".to_string(),
                input: Some(mapped_input.clone()),
                input_version: 1,
                result: None,
                result_version: 1,
                start_time: Utc::now().naive_utc(),
                close_time: None,
                current_event_id: 0,
                memo: None,
                search_attrs: None,
                context_snapshot: Some(mapped_input.clone()),
                version: 1,
                parent_run_id: Some(scope.run_id.to_string()),
                parent_state_name: Some(scope.state_name.to_string()),
                dsl_definition: Some(json!(state.iterator)),
            };

            scope
                .persistence
                .create_execution(&subflow)
                .await
                .map_err(|e| {
                    error!(%child_run_id, ?e, "❌ Failed to insert subflow");
                    format!("Failed to create subflow: {e}")
                })?;

            if status == "READY" {
                self.subflow_match
                    .notify_subflow_ready(
                        child_run_id,
                        scope.run_id.to_string(),
                        scope.state_name.to_string(),
                        json!(state.iterator),
                        mapped_input,
                    )
                    .await
                    .map_err(|e| {
                        error!(%scope.run_id, "❌ notify_subflow_ready failed: {}", e);
                        e
                    })?;
            }
        }

        Ok(StateExecutionResult {
            output: parent_context.clone(),
            next_state: None,
            should_continue: false,
            metadata: Some(json!({ "subflows": items.len() })),
            is_blocking: true,
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

        if let Some(failed) = subflows
            .iter()
            .find(|s| s.status == "FAILED" || s.status == "CANCELLED")
        {
            return Err(format!(
                "Subflow {} failed or cancelled (status = {})",
                failed.run_id, failed.status
            ));
        }

        let all_done = subflows.iter().all(|s| s.status == "COMPLETED");
        debug!(
            total = subflows.len(),
            all_done, "🔎 Subflow completion check"
        );

        if all_done {
            let pipeline = MappingPipeline {
                input_mapping: None,
                output_mapping: state.base.output_mapping.as_ref(),
            };

            let mut ctx = parent_context.clone();
            for sub in &subflows {
                let sub_result = sub.result.clone().unwrap_or(Value::Null);
                ctx = pipeline.apply_output(&sub_result, &ctx)?;
            }

            return Ok(StateExecutionResult {
                output: ctx,
                next_state: state.base.next.clone(),
                should_continue: true,
                metadata: None,
                is_blocking: false,
            });
        }

        if let Some(waiting) = subflows.iter().find(|s| s.status == "WAITING") {
            self.subflow_match
                .notify_subflow_ready(
                    waiting.run_id.clone(),
                    scope.run_id.to_string(),
                    scope.state_name.to_string(),
                    json!(state.iterator),
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
            should_continue: false,
            metadata: None,
            is_blocking: true,
        })
    }
}

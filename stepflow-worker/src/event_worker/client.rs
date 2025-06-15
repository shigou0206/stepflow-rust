use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use std::time::Instant;

use stepflow_common::config::StepflowConfig;
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_dto::dto::worker::{TaskDetails, TaskStatus};
use stepflow_eventbus::global::dispatch_event;
use stepflow_tool::core::registry::ToolRegistry;

pub async fn execute_task(
    _client: &Client,          
    _config: &StepflowConfig, 
    registry: &ToolRegistry,
    task: TaskDetails,
) -> Result<()> {
    let start = Instant::now();

    let exec_result = registry
        .execute(&task.tool_type, task.parameters.clone())
        .await;

    let (_status, output) = match exec_result {
        Ok(ok) => (TaskStatus::SUCCEEDED, ok.output),
        Err(e) => (TaskStatus::FAILED, json!({ "error": e.to_string() })),
    };

    dispatch_event(EngineEvent::TaskFinished {
        run_id: task.run_id,
        state_name: task.state_name,
        output,
    })
    .await;

    Ok(())
}
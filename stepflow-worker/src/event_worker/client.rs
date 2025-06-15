use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;
use std::time::Instant;
use stepflow_tool::core::registry::ToolRegistry;
use stepflow_dto::dto::worker::{TaskDetails, TaskStatus, UpdateRequest};
use stepflow_common::config::StepflowConfig;

pub async fn execute_task(
    client: &Client,
    config: &StepflowConfig,
    registry: &ToolRegistry,
    task: TaskDetails,
) -> Result<()> {
    let start = Instant::now();

    let result = registry.execute(&task.tool_type, task.parameters.clone()).await;
    let (status, result) = match result {
        Ok(ok) => (TaskStatus::SUCCEEDED, ok.output),
        Err(e) => (TaskStatus::FAILED, json!({ "error": e.to_string() })),
    };

    let payload = UpdateRequest {
        run_id: task.run_id,
        state_name: task.state_name,
        status,
        result,
        duration_ms: Some(start.elapsed().as_millis() as u64),
        task_id: None,
    };

    let url = format!("{}/update", config.gateway_server_url);
    client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .context("Failed to send update")?
        .error_for_status()
        .context("Update failed")?;

    Ok(())
}
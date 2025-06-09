use crate::config::WorkerConfig;
use stepflow_dto::dto::worker::*;
use stepflow_tool::core::registry::ToolRegistry;
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Instant;

pub async fn poll_for_task(
    client: &Client,
    config: &WorkerConfig,
) -> Result<Option<(String, TaskDetails)>> {
    let url = format!("{}/poll", config.gateway_server_url);
    let req = PollRequest {
        worker_id: config.worker_id.clone(),
        capabilities: vec!["http".into(), "shell".into(), "file".into()],
    };

    let res: PollResponse = client
        .post(&url)
        .json(&req)
        .send()
        .await
        .context("Failed to send poll request")?
        .error_for_status()
        .context("Poll HTTP error")?
        .json()
        .await
        .context("Invalid poll response")?;

    if res.has_task {
        let tool_type = res
            .tool_type
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_string();

        if tool_type.is_empty() {
            anyhow::bail!("Empty tool_type received");
        }

        Ok(Some((
            config.worker_id.clone(),
            TaskDetails {
                run_id: res.run_id.unwrap(),
                state_name: res.state_name.unwrap(),
                tool_type,
                parameters: res.input.unwrap_or_default(),
            },
        )))
    } else {
        Ok(None)
    }
}

#[derive(Debug)]
pub struct TaskDetails {
    pub run_id: String,
    pub state_name: String,
    pub tool_type: String,
    pub parameters: Value,
}

pub async fn execute_task(
    client: &Client,
    config: &WorkerConfig,
    registry: &ToolRegistry,
    task: TaskDetails,
) -> Result<()> {
    let start = Instant::now();

    // ✅ 执行工具（仅传 parameters）
    let result = registry.execute(&task.tool_type, task.parameters.clone()).await;
    println!("tool result: {:?}", result);

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
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use stepflow_tool::core::registry::ToolRegistry;
use stepflow_tool::tools::http::HttpTool;
use stepflow_tool::tools::shell::ShellTool;
use stepflow_tool::tools::file::FileTool;

const DEFAULT_GATEWAY_SERVER_URL: &str = "http://127.0.0.1:3000/v1/worker";
const DEFAULT_WORKER_ID: &str = "tool-worker-1";

#[derive(Debug, Clone)]
struct WorkerConfig {
    worker_id: String,
    gateway_server_url: String,
}

impl WorkerConfig {
    fn from_env() -> Result<Self> {
        let worker_id = env::var("WORKER_ID").unwrap_or_else(|_| DEFAULT_WORKER_ID.to_string());
        let gateway_server_url = env::var("GATEWAY_SERVER_URL").unwrap_or_else(|_| DEFAULT_GATEWAY_SERVER_URL.to_string());
        
        info!(
            worker_id = %worker_id,
            gateway_server_url = %gateway_server_url,
            "Tool worker configured"
        );
        Ok(Self { worker_id, gateway_server_url })
    }
}

#[derive(Debug, Deserialize)]
struct PollApiResponse {
    has_task: bool,
    run_id: Option<String>,
    state_name: Option<String>,
    tool_type: Option<String>,
    input: Option<Value>,
}

#[derive(Debug)]
struct TaskDetails {
    run_id: String,
    state_name: String,
    tool_type: String,
    input: Value,
}

#[derive(Debug, Serialize)]
struct TaskResult {
    run_id: String,
    state_name: String,
    status: String,
    result: Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(true)
        .init();

    let config = WorkerConfig::from_env().context("Failed to load worker configuration")?;
    
    // 创建工具注册表
    let mut registry = ToolRegistry::new();
    registry.register(HttpTool::new(None))?;
    registry.register(ShellTool::new(None))?;
    registry.register(FileTool::new(None))?;
    let registry = Arc::new(registry);

    info!(
        worker_id = %config.worker_id,
        tools = ?registry.list_tools(),
        "Tool registry initialized"
    );
    
    run_worker_loop(config, registry).await
}

async fn run_worker_loop(config: WorkerConfig, registry: Arc<ToolRegistry>) -> Result<()> {
    let client = Client::new();
    let mut backoff_seconds = 1u64;
    const MAX_BACKOFF_SECONDS: u64 = 60;

    info!(worker_id = %config.worker_id, "Tool worker loop starting...");

    loop {
        match poll_for_task(&client, &config).await {
            Ok(Some(task_details)) => {
                backoff_seconds = 1; // 重置退避时间
                info!(
                    worker_id = %config.worker_id,
                    run_id = %task_details.run_id,
                    state_name = %task_details.state_name,
                    tool_type = %task_details.tool_type,
                    "Polled new task"
                );
                
                if let Err(e) = execute_task(&client, &config, &registry, task_details).await {
                    error!(
                        worker_id = %config.worker_id,
                        error = ?e,
                        "Task execution failed"
                    );
                }
            }
            Ok(None) => {
                debug!(
                    worker_id = %config.worker_id,
                    sleep_duration_s = backoff_seconds,
                    "No task available, sleeping."
                );
                sleep(Duration::from_secs(backoff_seconds)).await;
                backoff_seconds = (backoff_seconds * 2).min(MAX_BACKOFF_SECONDS);
            }
            Err(e) => {
                warn!(
                    worker_id = %config.worker_id,
                    error = ?e,
                    sleep_duration_s = backoff_seconds,
                    "Polling error, sleeping."
                );
                sleep(Duration::from_secs(backoff_seconds)).await;
                backoff_seconds = (backoff_seconds * 2).min(MAX_BACKOFF_SECONDS);
            }
        }
    }
}

async fn poll_for_task(client: &Client, config: &WorkerConfig) -> Result<Option<TaskDetails>> {
    let poll_url = format!("{}/poll", config.gateway_server_url);
    debug!(worker_id = %config.worker_id, url = %poll_url, "Polling for task");

    let response = client
        .post(&poll_url)
        .json(&json!({
            "worker_id": config.worker_id,
            "capabilities": ["http", "shell", "file"] // 声明支持的工具类型
        }))
        .send()
        .await
        .context("Failed to send poll request")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_else(|_| "<Failed to read error body>".to_string());
        return Err(anyhow::anyhow!(
            "Polling request failed with status: {}. Body: {}", status, error_body
        ));
    }

    let poll_api_response = response
        .json::<PollApiResponse>()
        .await
        .context("Failed to deserialize poll response")?;

    if poll_api_response.has_task {
        Ok(Some(TaskDetails {
            run_id: poll_api_response.run_id.context("Task received with no run_id")?,
            state_name: poll_api_response.state_name.context("Task received with no state_name")?,
            tool_type: poll_api_response.tool_type.context("Task received with no tool_type")?,
            input: poll_api_response.input.unwrap_or_default(),
        }))
    } else {
        Ok(None)
    }
}

async fn execute_task(
    client: &Client,
    config: &WorkerConfig,
    registry: &ToolRegistry,
    task: TaskDetails
) -> Result<()> {
    info!(
        worker_id = %config.worker_id,
        run_id = %task.run_id,
        state_name = %task.state_name,
        tool_type = %task.tool_type,
        input = %serde_json::to_string_pretty(&task.input).unwrap(),
        "Executing task"
    );

    let start_time = std::time::Instant::now();
    debug!("Executing tool {} with input: {:?}", task.tool_type, task.input);
    let result = registry.execute(&task.tool_type, task.input).await;

    let (status, output) = match result {
        Ok(tool_result) => {
            info!(
                worker_id = %config.worker_id,
                run_id = %task.run_id,
                duration_ms = %start_time.elapsed().as_millis(),
                output = %serde_json::to_string_pretty(&tool_result.output).unwrap(),
                "Task executed successfully"
            );
            ("SUCCEEDED", tool_result.output)
        }
        Err(e) => {
            error!(
                worker_id = %config.worker_id,
                run_id = %task.run_id,
                error = ?e,
                "Task execution failed"
            );
            ("FAILED", json!({
                "error": e.to_string(),
                "duration_ms": start_time.elapsed().as_millis()
            }))
        }
    };

    let update_request = TaskResult {
        run_id: task.run_id,
        state_name: task.state_name,
        status: status.to_string(),
        result: output,
    };

    let update_url = format!("{}/update", config.gateway_server_url);
    debug!(worker_id = %config.worker_id, url = %update_url, "Sending task update");

    let response = client
        .post(&update_url)
        .json(&update_request)
        .send()
        .await
        .context("Failed to send task update request")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_else(|_| "<Failed to read error body>".to_string());
        return Err(anyhow::anyhow!(
            "Task update request failed with status: {}. Body: {}", status, error_body
        ));
    }

    info!(worker_id = %config.worker_id, url = %update_url, "Task update sent successfully");
    Ok(())
} 
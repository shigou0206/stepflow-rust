use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::{sync::Semaphore, time::sleep};
use tracing::{debug, error, info, warn};

use stepflow_tool::core::registry::ToolRegistry;
use stepflow_tool::tools::http::HttpTool;
use stepflow_tool::tools::shell::ShellTool;
use stepflow_tool::tools::file::FileTool;

const DEFAULT_GATEWAY_SERVER_URL: &str = "http://127.0.0.1:3000/v1/worker";
const DEFAULT_WORKER_ID: &str = "tool-worker";
const MAX_CONCURRENCY: usize = 4;

#[derive(Debug, Clone)]
struct WorkerConfig {
    worker_id: String,
    gateway_server_url: String,
}

impl WorkerConfig {
    fn from_env(index: usize) -> Result<Self> {
        let base_id = env::var("WORKER_ID").unwrap_or_else(|_| DEFAULT_WORKER_ID.to_string());
        let gateway_server_url = env::var("GATEWAY_SERVER_URL")
            .unwrap_or_else(|_| DEFAULT_GATEWAY_SERVER_URL.to_string());
        let worker_id = format!("{}-{}", base_id, index);
        Ok(Self {
            worker_id,
            gateway_server_url,
        })
    }
}

#[derive(Debug, Deserialize)]
struct PollApiResponse {
    has_task: bool,
    task: Option<InnerTask>,
}

#[derive(Debug, Deserialize)]
struct InnerTask {
    run_id: String,
    state_name: String,
    resource: String,
    task_payload: Option<Value>,
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

    let registry = Arc::new(init_registry()?);
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENCY));
    let client = Arc::new(Client::new());

    for i in 0..MAX_CONCURRENCY {
        let permit = semaphore.clone().acquire_owned().await?;
        let registry = registry.clone();
        let client = client.clone();
        let config = WorkerConfig::from_env(i)?;

        tokio::spawn(async move {
            let _permit = permit;
            loop {
                match poll_for_task(&client, &config).await {
                    Ok(Some(task)) => {
                        if let Err(e) = execute_task(&client, &config, &registry, task).await {
                            error!("❌ Task execution failed: {e:#}");
                        }
                    }
                    Ok(None) => {
                        debug!("No task available for {}", config.worker_id);
                        sleep(Duration::from_secs(2)).await;
                    }
                    Err(e) => {
                        warn!("Polling error: {e:#}");
                        sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        });
    }

    futures::future::pending::<()>().await;
    Ok(())
}

fn init_registry() -> Result<ToolRegistry> {
    let mut registry = ToolRegistry::new();
    registry.register(HttpTool::new(None))?;
    registry.register(ShellTool::new(None))?;
    registry.register(FileTool::new(None))?;
    Ok(registry)
}

async fn poll_for_task(client: &Client, config: &WorkerConfig) -> Result<Option<TaskDetails>> {
    let url = format!("{}/poll", config.gateway_server_url);
    let response = client
        .post(&url)
        .json(&json!({
            "worker_id": config.worker_id,
            "capabilities": ["http", "shell", "file"]
        }))
        .send()
        .await?
        .error_for_status()?;

    let res: PollApiResponse = response.json().await?;
    if let Some(task) = res.task {
        let tool_type = task.resource.trim();
        if tool_type.is_empty() {
            return Err(anyhow::anyhow!("Received task with empty resource/tool_type"));
        }
        Ok(Some(TaskDetails {
            run_id: task.run_id,
            state_name: task.state_name,
            tool_type: tool_type.to_string(),
            input: task.task_payload.unwrap_or_else(|| json!({})),
        }))
    } else {
        Ok(None)
    }
}

async fn execute_task(
    client: &Client,
    config: &WorkerConfig,
    registry: &ToolRegistry,
    task: TaskDetails,
) -> Result<()> {
    let start = std::time::Instant::now();
    info!("Executing task: run_id={}, tool={}", task.run_id, task.tool_type);

    let result = registry.execute(&task.tool_type, task.input.clone()).await;
    let (status, result) = match result {
        Ok(output) => ("SUCCEEDED", output.output),
        Err(e) => ("FAILED", json!({"error": e.to_string()})),
    };

    let payload = TaskResult {
        run_id: task.run_id,
        state_name: task.state_name,
        status: status.to_string(),
        result,
    };

    client
        .post(format!("{}/update", config.gateway_server_url))
        .json(&payload)
        .send()
        .await?
        .error_for_status()?;

    info!("📬 Task update sent: run_id={}, status={}", payload.run_id, payload.status);
    info!("⏱ Duration: {} ms", start.elapsed().as_millis());
    Ok(())
}
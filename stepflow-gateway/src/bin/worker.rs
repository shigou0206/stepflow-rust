// ⚙️ 修改版：支持并发轮询和任务执行控制
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::{sync::Semaphore, time::sleep};
use tracing::error;

use stepflow_tool::core::registry::ToolRegistry;
use stepflow_tool::tools::http::HttpTool;
use stepflow_tool::tools::shell::ShellTool;
use stepflow_tool::tools::file::FileTool;

const DEFAULT_GATEWAY_SERVER_URL: &str = "http://127.0.0.1:3000/v1/worker";
const DEFAULT_WORKER_ID: &str = "tool-worker-1";
const MAX_CONCURRENCY: usize = 4;

#[derive(Debug, Clone)]
struct WorkerConfig {
    worker_id: String,
    gateway_server_url: String,
}

impl WorkerConfig {
    fn from_env() -> Result<Self> {
        let worker_id = env::var("WORKER_ID").unwrap_or_else(|_| DEFAULT_WORKER_ID.to_string());
        let gateway_server_url = env::var("GATEWAY_SERVER_URL")
            .unwrap_or_else(|_| DEFAULT_GATEWAY_SERVER_URL.to_string());
        Ok(Self {
            worker_id,
            gateway_server_url,
        })
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

    let config = WorkerConfig::from_env()?;

    let mut registry = ToolRegistry::new();
    registry.register(HttpTool::new(None))?;
    registry.register(ShellTool::new(None))?;
    registry.register(FileTool::new(None))?;
    let registry = Arc::new(registry);

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENCY));
    let client = Arc::new(Client::new());

    loop {
        let permit = semaphore.clone().acquire_owned().await?;
        let config = config.clone();
        let registry = registry.clone();
        let client = client.clone();

        tokio::spawn(async move {
            let _permit = permit; // 控制并发
            if let Ok(Some(task)) = poll_for_task(&client, &config).await {
                if let Err(e) = execute_task(&client, &config, &registry, task).await {
                    error!("Worker task failed: {e:?}");
                }
            } else {
                sleep(Duration::from_secs(2)).await;
            }
        });

        sleep(Duration::from_millis(200)).await; // 控制轮询速率
    }
}

async fn poll_for_task(client: &Client, config: &WorkerConfig) -> Result<Option<TaskDetails>> {
    let res = client
        .post(format!("{}/poll", config.gateway_server_url))
        .json(&json!({
            "worker_id": config.worker_id,
            "capabilities": ["http", "shell", "file"]
        }))
        .send()
        .await?
        .json::<PollApiResponse>()
        .await?;

    if res.has_task {
        Ok(Some(TaskDetails {
            run_id: res.run_id.context("missing run_id")?,
            state_name: res.state_name.context("missing state_name")?,
            tool_type: res.tool_type.context("missing tool_type")?,
            input: res.input.unwrap_or_default(),
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
    let result = registry.execute(&task.tool_type, task.input.clone()).await;

    let (status, result) = match result {
        Ok(tool_result) => ("SUCCEEDED", tool_result.output),
        Err(e) => ("FAILED", json!({"error": e.to_string()})),
    };

    let res = TaskResult {
        run_id: task.run_id,
        state_name: task.state_name,
        status: status.to_string(),
        result,
    };

    client
        .post(format!("{}/update", config.gateway_server_url))
        .json(&res)
        .send()
        .await?;

    Ok(())
}
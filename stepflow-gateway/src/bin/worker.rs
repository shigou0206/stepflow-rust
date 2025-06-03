use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

const DEFAULT_GATEWAY_SERVER_URL: &str = "http://127.0.0.1:3000/v1/worker";
const DEFAULT_WORKER_ID: &str = "default-worker-1";

#[derive(Debug, Clone)]
struct WorkerConfig {
    worker_id: String,
    gateway_server_url: String, // Base URL for worker API, e.g., http://127.0.0.1:3000/v1/worker
}

impl WorkerConfig {
    fn from_env() -> Result<Self> {
        let worker_id = env::var("WORKER_ID").unwrap_or_else(|_| DEFAULT_WORKER_ID.to_string());
        let gateway_server_url = env::var("GATEWAY_SERVER_URL").unwrap_or_else(|_| DEFAULT_GATEWAY_SERVER_URL.to_string());
        
        info!(
            worker_id = %worker_id,
            gateway_server_url = %gateway_server_url,
            "Worker configured"
        );
        Ok(Self { worker_id, gateway_server_url })
    }
}

#[derive(Debug, Deserialize)]
struct PollApiResponse {
    has_task: bool,
    run_id: Option<String>,
    state_name: Option<String>,
    input: Option<Value>,
    // Potentially add task_type: Option<String> if gateway sends it
}

#[derive(Debug)]
struct TaskDetails {
    run_id: String,
    state_name: String,
    input: Value,
    // task_type: String, // If PollApiResponse includes it
}

#[derive(Debug, Serialize)]
struct TaskResult {
    run_id: String,
    state_name: String,
    status: String, // e.g., "SUCCEEDED", "FAILED"
    result: Value,  // Output of the task or error details
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(true)
        .init();

    let config = WorkerConfig::from_env().context("Failed to load worker configuration")?;
    
    run_worker_loop(config).await
}

async fn run_worker_loop(config: WorkerConfig) -> Result<()> {
    let client = Client::new();
    let mut backoff_seconds = 1u64;
    const MAX_BACKOFF_SECONDS: u64 = 60;

    info!(worker_id = %config.worker_id, "Worker loop starting...");

    loop {
        match poll_for_task(&client, &config).await {
            Ok(Some(task_details)) => {
                backoff_seconds = 1; // Reset backoff on successful task poll
                info!(worker_id = %config.worker_id, run_id = %task_details.run_id, state_name = %task_details.state_name, "Polled new task");
                if let Err(e) = execute_task(&client, &config, task_details).await {
                    error!(worker_id = %config.worker_id, error = ?e, "Task execution failed");
                }
            }
            Ok(None) => {
                debug!(worker_id = %config.worker_id, sleep_duration_s = backoff_seconds, "No task available, sleeping.");
                sleep(Duration::from_secs(backoff_seconds)).await;
                backoff_seconds = (backoff_seconds * 2).min(MAX_BACKOFF_SECONDS);
            }
            Err(e) => {
                warn!(worker_id = %config.worker_id, error = ?e, sleep_duration_s = backoff_seconds, "Polling error, sleeping.");
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
        .json(&json!({ "worker_id": config.worker_id }))
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
            input: poll_api_response.input.unwrap_or_default(),
        }))
    } else {
        Ok(None)
    }
}

async fn execute_task(client: &Client, config: &WorkerConfig, task: TaskDetails) -> Result<()> {
    info!(worker_id = %config.worker_id, run_id = %task.run_id, state_name = %task.state_name, "Executing task");

    // Simulate task execution (e.g., based on task.task_type if available)
    // For this example, we'll do the echo logic.
    let mut output_result = task.input.clone();
    if let Some(obj) = output_result.as_object_mut() {
        obj.insert("_worker_processed_by".to_string(), json!(config.worker_id));
        obj.insert("_worker_processed_at".to_string(), json!(chrono::Utc::now().to_rfc3339()));
    } else {
        // If input is not an object, wrap it or handle as appropriate
        output_result = json!({
            "original_input": task.input,
            "_worker_processed_by": config.worker_id,
            "_worker_processed_at": chrono::Utc::now().to_rfc3339()
        });
    }
    
    // Simulate some work
    sleep(Duration::from_secs(2)).await;

    let task_status = "SUCCEEDED"; // Can be "FAILED" on error
    info!(worker_id = %config.worker_id, run_id = %task.run_id, status = task_status, "Task execution finished");

    let update_request = TaskResult {
        run_id: task.run_id,
        state_name: task.state_name,
        status: task_status.to_string(),
        result: output_result,
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
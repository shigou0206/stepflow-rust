// use anyhow::{Context, Result};
// use reqwest::Client;
// use serde::{Deserialize, Serialize};
// use serde_json::{json, Value};
// use std::env;
// use std::sync::Arc;
// use std::time::Duration;
// use tokio::{sync::Semaphore, time::sleep};

// use stepflow_tool::core::registry::ToolRegistry;
// use stepflow_tool::tools::http::HttpTool;
// use stepflow_tool::tools::shell::ShellTool;
// use stepflow_tool::tools::file::FileTool;

// const DEFAULT_GATEWAY_SERVER_URL: &str = "http://127.0.0.1:3000/v1/worker";
// const DEFAULT_WORKER_ID: &str = "tool-worker";
// const MAX_CONCURRENCY: usize = 4;

// #[derive(Debug, Clone)]
// struct WorkerConfig {
//     worker_id: String,
//     gateway_server_url: String,
// }

// impl WorkerConfig {
//     fn from_env(index: usize) -> Result<Self> {
//         let base_id = env::var("WORKER_ID").unwrap_or_else(|_| DEFAULT_WORKER_ID.to_string());
//         let gateway_server_url = env::var("GATEWAY_SERVER_URL")
//             .unwrap_or_else(|_| DEFAULT_GATEWAY_SERVER_URL.to_string());
//         let worker_id = format!("{}-{}", base_id, index);
//         Ok(Self {
//             worker_id,
//             gateway_server_url,
//         })
//     }
// }

// #[derive(Debug, Deserialize)]
// struct PollApiResponse {
//     has_task: bool,
//     task: Option<InnerTask>,
// }

// #[derive(Debug, Deserialize)]
// struct InnerTask {
//     run_id: String,
//     state_name: String,
//     resource: String,
//     task_payload: Option<Value>,
// }

// #[derive(Debug)]
// struct TaskDetails {
//     run_id: String,
//     state_name: String,
//     tool_type: String,
//     input: Value,
// }

// #[derive(Debug, Serialize)]
// struct TaskResult {
//     run_id: String,
//     state_name: String,
//     status: String,
//     result: Value,
// }

// #[tokio::main]
// async fn main() -> Result<()> {
//     let registry = Arc::new(init_registry()?);
//     let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENCY));
//     let client = Arc::new(Client::new());

//     println!("🚀 Worker pool starting with concurrency = {}", MAX_CONCURRENCY);

//     for i in 0..MAX_CONCURRENCY {
//         let permit = semaphore.clone().acquire_owned().await?;
//         let registry = registry.clone();
//         let client = client.clone();
//         let config = WorkerConfig::from_env(i)?;

//         tokio::spawn(async move {
//             let _permit = permit;
//             loop {
//                 println!("[{}] 🔁 Poll loop tick", config.worker_id);
//                 match poll_for_task(&client, &config).await {
//                     Ok(Some(task)) => {
//                         println!(
//                             "[{}] 🎯 Task received: run_id={}, state={}, type={}",
//                             config.worker_id, task.run_id, task.state_name, task.tool_type
//                         );
//                         if let Err(e) = execute_task(&client, &config, &registry, task).await {
//                             eprintln!("[{}] ❌ Task execution error: {:#}", config.worker_id, e);
//                         }
//                     }
//                     Ok(None) => {
//                         println!("[{}] 💤 No task available", config.worker_id);
//                         sleep(Duration::from_secs(2)).await;
//                     }
//                     Err(e) => {
//                         eprintln!("[{}] ❌ Polling failed: {:#}", config.worker_id, e);
//                         sleep(Duration::from_secs(2)).await;
//                     }
//                 }
//             }
//         });
//     }

//     futures::future::pending::<()>().await;
//     Ok(())
// }

// fn init_registry() -> Result<ToolRegistry> {
//     let mut registry = ToolRegistry::new();
//     registry.register(HttpTool::new(None))?;
//     registry.register(ShellTool::new(None))?;
//     registry.register(FileTool::new(None))?;
//     println!("🧰 Registered tools: {:?}", registry.list_tools());
//     Ok(registry)
// }

// async fn poll_for_task(client: &Client, config: &WorkerConfig) -> Result<Option<TaskDetails>> {
//     let url = format!("{}/poll", config.gateway_server_url);
//     println!("[{}] 🔄 Polling from {}", config.worker_id, url);

//     let response = client
//         .post(&url)
//         .json(&json!({
//             "worker_id": config.worker_id,
//             "capabilities": ["http", "shell", "file"]
//         }))
//         .send()
//         .await
//         .context("❌ Failed to send poll request")?
//         .error_for_status()
//         .context("❌ Poll HTTP error")?;

//     let res: PollApiResponse = response.json().await.context("❌ Invalid poll JSON")?;
//     println!("[{}] 🧾 Poll response: {:?}", config.worker_id, res);

//     if res.has_task {
//         let task = res.task.context(format!(
//             "[{}] 🚫 Poll inconsistency: has_task=true but task is None", config.worker_id
//         ))?;
    
//         let tool_type = task.resource.trim();
//         if tool_type.is_empty() {
//             return Err(anyhow::anyhow!("[{}] 🚫 Task resource is empty", config.worker_id));
//         }
    
//         Ok(Some(TaskDetails {
//             run_id: task.run_id,
//             state_name: task.state_name,
//             tool_type: tool_type.to_string(),
//             input: task.task_payload.unwrap_or_else(|| json!({})),
//         }))
//     } else {
//         println!("[{}] 💤 No task available", config.worker_id);
//         Ok(None)
//     }

//     // if let Some(task) = res.task {
//     //     let tool_type = task.resource.trim();
//     //     if tool_type.is_empty() {
//     //         return Err(anyhow::anyhow!("[{}] 🚫 Task resource is empty", config.worker_id));
//     //     }
//     //     Ok(Some(TaskDetails {
//     //         run_id: task.run_id,
//     //         state_name: task.state_name,
//     //         tool_type: tool_type.to_string(),
//     //         input: task.task_payload.unwrap_or_else(|| json!({})),
//     //     }))
//     // } else {
//     //     Ok(None)
//     // }
// }

// async fn execute_task(
//     client: &Client,
//     config: &WorkerConfig,
//     registry: &ToolRegistry,
//     task: TaskDetails,
// ) -> Result<()> {
//     let start = std::time::Instant::now();
//     println!(
//         "[{}] 🚀 Running task: run_id={} state={} type={}",
//         config.worker_id, task.run_id, task.state_name, task.tool_type
//     );

//     println!(
//         "[{}] 📥 Input: {}",
//         config.worker_id,
//         serde_json::to_string_pretty(&task.input).unwrap_or_default()
//     );

//     let result = registry.execute(&task.tool_type, task.input.clone()).await;

//     let (status, output) = match result {
//         Ok(ok) => {
//             println!("[{}] ✅ Task succeeded", config.worker_id);
//             ("SUCCEEDED", ok.output)
//         }
//         Err(e) => {
//             eprintln!("[{}] ❌ Tool error: {}", config.worker_id, e);
//             ("FAILED", json!({ "error": e.to_string() }))
//         }
//     };

//     let payload = TaskResult {
//         run_id: task.run_id.clone(),
//         state_name: task.state_name.clone(),
//         status: status.to_string(),
//         result: output,
//     };

//     let url = format!("{}/update", config.gateway_server_url);
//     println!(
//         "[{}] 📤 Sending update: {}",
//         config.worker_id,
//         serde_json::to_string_pretty(&payload).unwrap_or_default()
//     );

//     match client.post(&url).json(&payload).send().await {
//         Ok(resp) if resp.status().is_success() => {
//             println!(
//                 "[{}] 📬 Update OK: run_id={}, status={}, duration={}ms",
//                 config.worker_id,
//                 task.run_id,
//                 status,
//                 start.elapsed().as_millis()
//             );
//         }
//         Ok(resp) => {
//             let code = resp.status();
//             let body = resp.text().await.unwrap_or_default();
//             eprintln!(
//                 "[{}] ❌ Update HTTP error: {} body={} payload={:?}",
//                 config.worker_id, code, body, payload
//             );
//         }
//         Err(e) => {
//             eprintln!(
//                 "[{}] ❌ Update failed: {} payload={:?}",
//                 config.worker_id, e, payload
//             );
//         }
//     }

//     Ok(())
// }
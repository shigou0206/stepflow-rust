use crate::{client, config::WorkerConfig};
use anyhow::Result;
use reqwest::Client;
use stepflow_tool::{core::registry::ToolRegistry, tools::{http::HttpTool, shell::ShellTool, file::FileTool}};
use std::sync::Arc;
use tokio::{sync::Semaphore, time::sleep};
use std::time::Duration;

const MAX_CONCURRENCY: usize = 1;

pub async fn start() -> Result<()> {
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
                match client::poll_for_task(&client, &config).await {
                    Ok(Some((worker_id, task))) => {
                        println!("[{worker_id}] Task received: {}", task.state_name);
                        if let Err(e) = client::execute_task(&client, &config, &registry, task).await {
                            eprintln!("[{worker_id}] Task execution error: {e:#}");
                        }
                    }
                    Ok(None) => {
                        println!("[{}] No task available", config.worker_id);
                        sleep(Duration::from_secs(2)).await;
                    }
                    Err(e) => {
                        eprintln!("[{}] Polling error: {e:#}", config.worker_id);
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
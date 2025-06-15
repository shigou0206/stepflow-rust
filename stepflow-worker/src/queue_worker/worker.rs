use crate::queue_worker::client;
use stepflow_common::config::StepflowConfig;
use anyhow::Result;
use reqwest::Client;
use stepflow_tool::core::registry::ToolRegistry;
use std::sync::Arc;
use tokio::{sync::Semaphore, time::sleep};
use std::time::Duration;

/// 启动基于轮询的 Worker
pub async fn start_queue_worker(
    config: StepflowConfig,
    client: Arc<Client>,
    registry: Arc<ToolRegistry>,
    concurrency: usize,
) -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(concurrency));

    for _i in 0..concurrency {
        let permit = semaphore.clone().acquire_owned().await?;
        let registry = registry.clone();
        let client = client.clone();
        let config = config.clone(); // 支持多实例独立 ID，如 worker-0, worker-1

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
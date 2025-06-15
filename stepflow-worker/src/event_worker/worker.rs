use crate::config::WorkerConfig;
use anyhow::Result;
use reqwest::Client;
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_dto::dto::worker::TaskDetails;
use stepflow_eventbus::core::bus::EventBus;
use stepflow_tool::core::registry::ToolRegistry;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;

use crate::event_worker::client::execute_task;

/// 启动一个事件驱动的 Worker
pub async fn start_event_worker(
    config: WorkerConfig,
    client: Arc<Client>,
    registry: Arc<ToolRegistry>,
    bus: Arc<dyn EventBus>,
    concurrency: usize,
) -> Result<()> {
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut rx = bus.subscribe();

    while let Ok(envelope) = rx.recv().await {
        if let EngineEvent::TaskReady {
            run_id,
            state_name,
            resource,
            input,
        } = envelope.payload
        {
            if !config.capabilities.contains(&resource) {
                continue;
            }

            // 构造 TaskDetails
            let task = TaskDetails {
                run_id,
                state_name,
                tool_type: resource,
                parameters: input.unwrap_or_default(),
            };

            // 控制并发
            let permit = match semaphore.clone().try_acquire_owned() {
                Ok(p) => p,
                Err(_) => {
                    tracing::warn!("[{}] Worker is saturated, skipping task", config.worker_id);
                    continue;
                }
            };

            // 克隆必要变量用于 tokio::spawn
            let client = client.clone();
            let config = config.clone();
            let registry = registry.clone();

            tokio::spawn(async move {
                let _permit = permit;
                let task_id = Uuid::new_v4();
                tracing::info!("🔧 Task {} executing: {}.{}", task_id, task.run_id, task.state_name);

                if let Err(e) = execute_task(&client, &config, &registry, task).await {
                    tracing::error!("❌ Task execution failed: {e:#}");
                }
            });
        }
    }

    Ok(())
}
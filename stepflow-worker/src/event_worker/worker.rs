use anyhow::Result;
use reqwest::Client;
use stepflow_dto::dto::engine_event::EngineEvent;
use stepflow_dto::dto::worker::TaskDetails;
use stepflow_eventbus::core::bus::EventBus;
use stepflow_tool::core::registry::ToolRegistry;
use std::sync::Arc;
use tokio::sync::Semaphore;
use uuid::Uuid;
use stepflow_common::config::StepflowConfig;
use crate::event_worker::client::execute_task;

pub async fn start_event_worker(
    config: StepflowConfig,
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

            let task = TaskDetails {
                run_id,
                state_name,
                tool_type: resource,
                parameters: input.unwrap_or_default(),
            };

            // ✅ 等待可用许可，而不是跳过任务
            let permit = semaphore.clone().acquire_owned().await?;

            // 克隆必要变量
            let client = client.clone();
            let config = config.clone();
            let registry = registry.clone();

            tokio::spawn(async move {
                let _permit = permit;
                let task_id = Uuid::new_v4();
                tracing::info!(
                    "🔧 Task {} executing: {}.{}",
                    task_id,
                    task.run_id,
                    task.state_name
                );

                if let Err(e) = execute_task(&client, &config, &registry, task).await {
                    tracing::error!("❌ Task execution failed: {e:#}");
                }
            });
        }
    }

    Ok(())
}
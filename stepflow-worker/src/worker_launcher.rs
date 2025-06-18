use anyhow::Result;
use std::sync::Arc;
use reqwest::Client;

use stepflow_common::config::{StepflowConfig, StepflowExecMode};
use stepflow_tool::registry::globals::GLOBAL_TOOL_REGISTRY;
use stepflow_eventbus::global::get_global_event_bus;
use crate::{start_queue_worker, start_event_worker};

pub async fn launch_worker() -> Result<()> {
    // 加载配置
    let config = StepflowConfig::from_env(0)?;
    let concurrency = config.concurrency;

    // 注入依赖
    let client = Arc::new(Client::new());
    let registry = GLOBAL_TOOL_REGISTRY.clone();

    match config.exec_mode {
        StepflowExecMode::Polling => {
            tracing::info!("🚀 Starting in polling mode... {}", config.summary());
            start_queue_worker(config, client, registry, concurrency).await?;
        }
        StepflowExecMode::EventDriven => {
            tracing::info!("🚀 Starting in event-driven mode... {}", config.summary());
            let bus = get_global_event_bus().cloned().expect("GLOBAL_EVENT_BUS not set");
            start_event_worker(config, client, registry, bus, concurrency).await?;
        }
    }

    Ok(())
}
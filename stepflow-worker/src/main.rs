use anyhow::Result;

use stepflow_common::config::{StepflowConfig, StepflowMode};
use tracing_subscriber::EnvFilter;
use std::sync::Arc;
use reqwest::Client;

use stepflow_worker::{
    start_queue_worker,
    start_event_worker,
};
use stepflow_tool::registry::globals::GLOBAL_TOOL_REGISTRY;
use stepflow_eventbus::impls::local::LocalEventBus;

const DEFAULT_CONCURRENCY: usize = 1;

#[tokio::main]
async fn main() -> Result<()> {
    // —— 日志初始化 ——
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("debug"))
        .with_target(true)
        .init();

    // —— 环境配置 ——
    let config = StepflowConfig::from_env(0)?;
    let concurrency = DEFAULT_CONCURRENCY;

    // —— 公共依赖注入 ——
    let client = Arc::new(Client::new());
    let registry = GLOBAL_TOOL_REGISTRY.clone();

    match config.mode {
        StepflowMode::Polling => {
            tracing::info!("🚀 Starting in polling mode...");
            start_queue_worker(config, client, registry, concurrency).await?;
        }
        StepflowMode::EventDriven => {
            tracing::info!("🚀 Starting in event-driven mode...");
            let bus = Arc::new(LocalEventBus::new(100)); // 后期可注入远程 EventBus
            start_event_worker(config, client, registry, bus, concurrency).await?;
        }
    }

    Ok(())
}
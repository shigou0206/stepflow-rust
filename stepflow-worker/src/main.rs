use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("debug")) // ✅ 全局 debug 等级
        .with_target(true)
        .init();

    stepflow_worker::start().await
}

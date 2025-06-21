pub mod app_state;
pub mod error;
pub mod builder;
pub mod event_runner;
pub mod service;
pub mod service_register;
pub mod event_handlers;
pub mod watcher;
pub mod poller_manager;
pub use app_state::AppState;
pub use error::{AppError, AppResult};

/// 初始化 tracing 日志系统
pub fn init_tracing() -> anyhow::Result<()> {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,sqlx::query=off")
    });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();

    Ok(())
}
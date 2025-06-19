pub mod app_state;
pub mod error;
pub mod builder;
pub mod event;
pub mod service;

pub use app_state::AppState;
pub use error::{AppError, AppResult};

/// 初始化 tracing 日志系统
pub fn init_tracing() -> anyhow::Result<()> {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("debug".parse()?))
        .with_target(true)
        .init();
    Ok(())
}
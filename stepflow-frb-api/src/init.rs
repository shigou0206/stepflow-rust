pub use stepflow_core::app_state::AppState;
pub use stepflow_core::service::execution::ExecutionSqlxSvc;

use std::sync::Arc;

use flutter_rust_bridge::frb;
use once_cell::sync::OnceCell;

use stepflow_common::config::{StepflowConfig, StepflowExecMode};
use stepflow_core::{
    builder::build_app_state,
    event_runner::{maybe_start_event_runner, spawn_event_logger},
    init_tracing,
};

use stepflow_eventbus::global::set_global_event_bus;

/// 全局共享 AppState
pub static GLOBAL_APP_STATE: OnceCell<Arc<AppState>> = OnceCell::new();
pub static EXECUTION_SVC: OnceCell<ExecutionSqlxSvc> = OnceCell::new();

#[frb]
pub async fn init_stepflow() -> anyhow::Result<()> {
    // ① 初始化日志
    init_tracing()?;

    // ② 加载配置并校验模式
    let config = StepflowConfig::for_flutter()?;
    if config.exec_mode != StepflowExecMode::EventDriven {
        return Err(anyhow::anyhow!("❌ FRB 模式必须使用 event-driven 执行模式"));
    }

    // ③ 构建 AppState + 设置全局 EventBus
    let app_state = Arc::new(build_app_state(&config).await?);
    set_global_event_bus(app_state.event_bus.clone())?;

    // ④ 启动引擎监听器 + 日志监听器
    maybe_start_event_runner(&config, &app_state);
    spawn_event_logger(&app_state);

    // ⑤ 注册全局状态
    GLOBAL_APP_STATE
        .set(app_state.clone())
        .map_err(|_| anyhow::anyhow!("Stepflow 已初始化"))?;

    let execution_svc = ExecutionSqlxSvc::new(app_state.clone());

    EXECUTION_SVC
        .set(execution_svc)
        .map_err(|_| anyhow::anyhow!("ExecutionSqlxSvc 已初始化"))?;

    // ✅ 可选：直接启动 Worker（默认开启）
    tokio::spawn(async move {
        if let Err(e) = stepflow_worker::launch_worker().await {
            tracing::error!("❌ Local worker exited: {e:#}");
        }
    });

    Ok(())
}

#[cfg(not(frb_expand))]
pub fn get_app_state() -> Arc<AppState> {
    GLOBAL_APP_STATE.get().expect("Stepflow 尚未初始化").clone()
}

#[cfg(not(frb_expand))]
pub fn get_execution_svc() -> &'static ExecutionSqlxSvc {
    EXECUTION_SVC
        .get()
        .expect("ExecutionSqlxSvc not initialized")
}

mod routes;

use routes::ApiDoc;

use std::net::SocketAddr;
use axum::Router;
use stepflow_common::config::StepflowConfig;
use stepflow_core::{
    builder::build_app_state, 
    app_state::AppState, 
    event::{maybe_start_event_runner, spawn_event_logger},
    init_tracing
};
use stepflow_eventbus::global::set_global_event_bus;
use stepflow_worker::launch_worker;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ① 初始化日志
    init_tracing()?;

    // ② 加载配置并构建 AppState
    let config = StepflowConfig::from_env_default()?;
    let app_state = build_app_state(&config).await?;
    set_global_event_bus(app_state.event_bus.clone())?;

    // ③ 启动 EventRunner（如启用）+ 日志监听器
    maybe_start_event_runner(&config, &app_state);
    spawn_event_logger(&app_state);

    // ④ 启动 HTTP + Worker 服务
    run_gateway_server(config, app_state).await
}

/// 构建 Axum HTTP 服务
fn build_http_router(app_state: AppState) -> Router<AppState> {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(routes::new(app_state.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
}

/// 启动 HTTP + Worker 并发服务
async fn run_gateway_server(config: StepflowConfig, app_state: AppState) -> anyhow::Result<()> {
    let addr: SocketAddr = config.gateway_bind.parse().unwrap_or(([127, 0, 0, 1], 3000).into());
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("🚀 Gateway listening at http://{}", addr);

    let app = build_http_router(app_state.clone());

    tokio::select! {
        res = axum::serve(listener, app.with_state(app_state.clone())) => {
            if let Err(e) = res {
                tracing::error!("❌ HTTP server exited: {e:#}");
            }
        }
        res = launch_worker() => {
            if let Err(e) = res {
                tracing::error!("❌ Worker exited: {e:#}");
            }
        }
    }

    Ok(())
}
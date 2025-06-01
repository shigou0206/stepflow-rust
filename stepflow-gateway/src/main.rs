mod app_state;
mod error;
mod middleware;
mod dto;
mod service;
mod routes;

use axum::Router;
use tracing_subscriber::EnvFilter;
use app_state::AppState;
use stepflow_storage::PersistenceManagerImpl;
use stepflow_hook::{EngineEventDispatcher, impls::log_hook::LogHook};
use tower_http::{
    trace::TraceLayer,
    cors::CorsLayer,
    compression::CompressionLayer,
};
use std::net::SocketAddr;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        routes::template::create,
        routes::template::list,
        routes::template::get_one,
        routes::template::update,
        routes::template::delete_one,
        routes::execution::start,
        routes::execution::list,
        routes::execution::get_one,
        routes::worker::poll_task,
        routes::activity_task::list_tasks,
        routes::activity_task::get_task,
        routes::activity_task::get_tasks_by_run_id,
        routes::activity_task::start_task,
        routes::activity_task::complete_task,
        routes::activity_task::fail_task,
        routes::activity_task::heartbeat_task,
    ),
    components(
        schemas(
            dto::template::TemplateDto,
            dto::template::TemplateUpsert,
            dto::execution::ExecStart,
            dto::execution::ExecDto,
            dto::worker::PollRequest,
            dto::worker::PollResponse,
            dto::activity_task::ListQuery,
            dto::activity_task::ActivityTaskDto,
            dto::activity_task::CompleteRequest,
            dto::activity_task::FailRequest,
            dto::activity_task::HeartbeatRequest,
        )
    ),
    tags(
        (name = "templates", description = "工作流模板管理"),
        (name = "executions", description = "工作流执行管理"),
        (name = "worker", description = "Worker 任务管理"),
        (name = "activity_tasks", description = "活动任务管理"),
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // -------- log ----------
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("stepflow_gateway=debug".parse()?))
        .init();

    // -------- infra (stub) --
    let pool = sqlx::SqlitePool::connect_lazy("sqlite:/Users/sryu/projects/stepflow-rust/stepflow-sqlite/data/data.db")?;
    let persist = std::sync::Arc::new(PersistenceManagerImpl::new(pool.clone()));
    let event_dispatcher = std::sync::Arc::new(EngineEventDispatcher::new(vec![LogHook::new()]));

    let state = AppState { 
        pool, 
        persist, 
        engines: Default::default(),
        event_dispatcher,
    };

    // -------- router -------
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(routes::new(state.clone()))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("🚀 gateway listen on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.with_state(state)).await?;

    Ok(())
}
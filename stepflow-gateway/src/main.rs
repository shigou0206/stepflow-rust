mod app_state;
mod error;
mod service;
mod routes;

use axum::Router;
use tracing_subscriber::EnvFilter;
use std::sync::Arc;
use app_state::AppState;
use stepflow_sqlite::SqliteStorageManager;
use stepflow_hook::{EngineEventDispatcher, impls::log_hook::LogHook};
use stepflow_match::service::{MemoryMatchService, HybridMatchService, PersistentMatchService};
use stepflow_match::queue::PersistentStore;
use sqlx::SqlitePool;
use tower_http::{
    trace::TraceLayer,
    cors::CorsLayer,
    compression::CompressionLayer,
};
use std::net::SocketAddr;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use stepflow_dto::dto as dto;
use std::str::FromStr;

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
        routes::execution::update,
        routes::execution::delete_one,
        routes::execution::list_by_status,
        routes::worker::poll_task,
        routes::worker::update_task_status,
        routes::activity_task::list_tasks,
        routes::activity_task::get_task,
        routes::activity_task::get_tasks_by_run_id,
        routes::activity_task::start_task,
        routes::activity_task::complete_task,
        routes::activity_task::fail_task,
        routes::activity_task::heartbeat_task,
        routes::workflow_event::list_events,
        routes::workflow_event::get_event,
        routes::workflow_event::record_event,
        routes::workflow_event::archive_event,
        routes::workflow_event::delete_event,
        routes::queue_task::get_one,
        routes::queue_task::update_one,
        routes::queue_task::list_by_status,
        routes::queue_task::validate_dsl,
        routes::queue_task::list_to_retry,
        routes::queue_task::delete_one,
        routes::timer::create_timer,
        routes::timer::get_timer,
        routes::timer::update_timer,
        routes::timer::delete_timer,
        routes::timer::find_before,
        routes::match_router::enqueue_task,
        routes::match_router::poll_task,
        routes::match_router::get_stats,
    ),
    components(
        schemas(
            dto::template::TemplateDto,
            dto::template::TemplateUpsert,
            dto::execution::ExecStart,
            dto::execution::ExecDto,
            dto::worker::PollRequest,
            dto::worker::PollResponse,
            dto::worker::UpdateRequest,
            dto::activity_task::ListQuery,
            dto::activity_task::ActivityTaskDto,
            dto::activity_task::CompleteRequest,
            dto::activity_task::FailRequest,
            dto::activity_task::HeartbeatRequest,
            dto::workflow_event::ListQuery,
            dto::workflow_event::WorkflowEventDto,
            dto::workflow_event::RecordEventRequest,
            dto::queue_task::QueueTaskDto,
            dto::queue_task::UpdateQueueTaskDto,
            dto::timer::TimerDto,
            dto::timer::CreateTimerDto,
            dto::timer::UpdateTimerDto,
            dto::match_stats::EnqueueRequest,
            dto::match_stats::PollRequest,
            dto::match_stats::PollResponse,
            dto::match_stats::MatchStats,
        )
    ),
    tags(
        (name = "templates", description = "工作流模板管理"),
        (name = "executions", description = "工作流执行管理"),
        (name = "worker", description = "Worker 任务管理"),
        (name = "activity_tasks", description = "活动任务管理"),
        (name = "workflow_events", description = "工作流事件管理"),
        (name = "queue_tasks", description = "队列任务管理"),
        (name = "timers", description = "定时任务管理"),
        (name = "match", description = "匹配服务管理"),
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // -------- log ----------
    tracing_subscriber::fmt()
    .with_env_filter(
        EnvFilter::from_default_env()
            .add_directive("debug".parse()?) // ✅ 启用全局 debug
    )
    .with_target(true) // ✅ 打印模块名
    .init();

    // -------- infra (stub) --
    // let db_url = concat!(
    //     "sqlite:/Users/sryu/projects/stepflow-rust/stepflow-sqlite/data/data.db",
    //     "?mode=rwc",               // 读写并创建
    //     "&journal_mode=WAL",       // 打开 WAL
    //     "&busy_timeout=5000"       // 冲突时最多等 5 s
    // );
    
    let db_options = SqliteConnectOptions::from_str("sqlite:/Users/sryu/projects/stepflow-rust/stepflow-sqlite/data/data.db")?
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal) // ← 设置 WAL 模式
    .busy_timeout(std::time::Duration::from_secs(5)); // ← 设置 busy_timeout

    let pool = SqlitePool::connect_with(db_options).await?;
    let persist = std::sync::Arc::new(SqliteStorageManager::new(pool.clone()));
    let event_dispatcher = std::sync::Arc::new(EngineEventDispatcher::new(vec![LogHook::new()]));
    // let match_service = MemoryMatchService::new();

    let memory_match = MemoryMatchService::new();
    let persistent_store = Arc::new(PersistentStore::new(persist.clone()));
    let persistent_match = PersistentMatchService::new(persistent_store.clone(), persist.clone());

    let match_service = HybridMatchService::new(
        memory_match.clone(),     // memory 组件
        persistent_match.clone(), // persistent 组件
    );

    let state = AppState { 
        persist, 
        engines: Default::default(),
        event_dispatcher,
        match_service,
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
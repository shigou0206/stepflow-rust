mod routes;
mod service;
use stepflow_core::app_state::AppState;
use axum::Router;
use prometheus::Registry;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use stepflow_dto::dto;
use stepflow_eventbus::impls::local::LocalEventBus;
use stepflow_hook::{
    EngineEventDispatcher, impls::log_hook::LogHook, impls::metrics_hook::MetricsHook,
    impls::persist_hook::PersistHook,
};
use stepflow_match::queue::PersistentStore;
use stepflow_match::service::{HybridMatchService, MemoryMatchService, PersistentMatchService};
use stepflow_sqlite::SqliteStorageManager;
use stepflow_storage::traits::{EventStorage, StateStorage, WorkflowStorage};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use stepflow_engine::handler::{
    choice::ChoiceHandler, fail::FailHandler, pass::PassHandler, registry::StateHandlerRegistry,
    succeed::SucceedHandler, task::TaskHandler, wait::WaitHandler,
};

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
            EnvFilter::from_default_env().add_directive("debug".parse()?), // ✅ 启用全局 debug
        )
        .with_target(true) // ✅ 打印模块名
        .init();

    let db_path = PathBuf::from("data/stepflow.db");
    let db_url = format!("sqlite://{}", db_path.to_string_lossy());

    let db_options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePool::connect_with(db_options).await?;
    let persist = std::sync::Arc::new(SqliteStorageManager::new(pool.clone()));

    // --- Event Dispatcher & Hooks ---
    let workflow_store: Arc<dyn WorkflowStorage> = persist.clone();
    let state_store: Arc<dyn StateStorage> = persist.clone();
    let event_store: Arc<dyn EventStorage> = persist.clone();

    let event_bus = Arc::new(LocalEventBus::new(100));

    let mut event_dispatcher = EngineEventDispatcher::new(
        vec![
            LogHook::new(),
            MetricsHook::new(&Registry::new()),
            PersistHook::new(
                workflow_store.clone(), // impl WorkflowStorage
                state_store.clone(),    // impl StateStorage
                event_store.clone(),    // impl EventStorage
            ),
            // 如果有 WebSocket 推送：
            // WsHook::new(ws_sender.clone()),
        ],
        event_bus.clone(),
    );
    // 可选：批量异步刷写，每秒或 100 条
    event_dispatcher = event_dispatcher.enable_batch_processing(100, Duration::from_secs(1));
    let event_dispatcher = Arc::new(event_dispatcher);

    // let event_dispatcher = std::sync::Arc::new(EngineEventDispatcher::new(vec![LogHook::new()]));
    // let match_service = MemoryMatchService::new();

    let memory_match = MemoryMatchService::new();
    let persistent_store = Arc::new(PersistentStore::new(persist.clone()));
    let persistent_match = PersistentMatchService::new(persistent_store.clone(), persist.clone());

    let match_service = HybridMatchService::new(
        memory_match.clone(),     // memory 组件
        persistent_match.clone(), // persistent 组件
    );

    let state_handler_registry = StateHandlerRegistry::new()
        .register("task", Arc::new(TaskHandler::new(match_service.clone())))
        .register("wait", Arc::new(WaitHandler::new()))
        .register("pass", Arc::new(PassHandler::new()))
        .register("choice", Arc::new(ChoiceHandler::new()))
        .register("succeed", Arc::new(SucceedHandler::new()))
        .register("fail", Arc::new(FailHandler::new()));
    let state_handler_registry = Arc::new(state_handler_registry);

    let state = AppState {
        persist,
        engines: Default::default(),
        event_dispatcher,
        match_service,
        event_bus,
        state_handler_registry,
    };

    let mut bus_rx = state.subscribe_events();
    tokio::spawn(async move {
        while let Ok(envelope) = bus_rx.recv().await {
            tracing::debug!(event = ?envelope, "🔔 got envelope from EventBus");

            // —— 交给前端桥接 ——
            // ui_bridge::push_to_flutter(envelope.clone()).await;

            // —— 也可以交给 SignalManager、监控、不阻塞……
            // state.signal_manager.handle_envelope(envelope.clone()).await;
        }
        tracing::warn!("🚧 EventBus subscription closed");
    });

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

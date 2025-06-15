mod routes;
mod service;
use stepflow_core::app_state::AppState;
use stepflow_core::builder::build_app_state;
use axum::Router;
use std::net::SocketAddr;
use stepflow_dto::dto;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use stepflow_common::config::StepflowConfig;


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
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("debug".parse()?))
        .with_target(true)
        .init();

    // ① 加载配置
    let config = StepflowConfig::from_env_default()?;

    // ② 构造 AppState
    let app_state: AppState = build_app_state(&config).await?;

    // ③ 启动事件总线监听
    let mut bus_rx = app_state.subscribe_events();
    tokio::spawn({
        let _state = app_state.clone();
        async move {
            while let Ok(envelope) = bus_rx.recv().await {
                tracing::debug!(?envelope, "🔔 Got EventEnvelope from EventBus");
                // 这里可以调用 ui_bridge / signal_manager / metrics 分发等
            }
            tracing::warn!("⚠️ EventBus subscription closed");
        }
    });

    let cloned_state = app_state.clone();

    // ④ 启动 HTTP 服务
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(routes::new(app_state))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new());

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        tracing::info!("🚀 gateway listen on http://{}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app.with_state(cloned_state)).await?;

    Ok(())
}
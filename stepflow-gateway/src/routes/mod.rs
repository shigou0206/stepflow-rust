use axum::{Router, routing::get};
pub mod template;
pub mod execution;
pub mod worker;
pub mod activity_task;
pub mod workflow_event;
pub mod queue_task;
pub mod timer;
pub mod match_router;
use crate::{
    service::{
        template::TemplateSqlxSvc,
        execution::ExecutionSqlxSvc,
        activity_task::ActivityTaskSqlxSvc,
        workflow_event::WorkflowEventSqlxSvc,
        queue_task::QueueTaskSqlxSvc,
        timer::TimerSqlxSvc
    },
};
use stepflow_core::app_state::AppState;
use std::sync::Arc;

pub fn new(state: AppState) -> Router<AppState> {
    let state = Arc::new(state);
    // 创建要用的 ServiceImpl
    let tpl_svc = TemplateSqlxSvc::new(state.persist.clone());
    let exec_svc = ExecutionSqlxSvc::new(state.clone());
    let event_svc = WorkflowEventSqlxSvc::new(state.persist.clone());
    let task_svc = ActivityTaskSqlxSvc::new(state.persist.clone());
    let queue_svc = QueueTaskSqlxSvc::new(state.clone());
    let timer_svc = TimerSqlxSvc::new(state.clone());


    let app = Router::new()
        .nest("/v1/templates", template::router(tpl_svc))
        .nest("/v1/executions", execution::router(exec_svc))
        .nest("/v1/activity_tasks", activity_task::router(task_svc))
        .nest("/v1/worker", worker::router())
        .nest("/v1/workflow_events", workflow_event::router(event_svc))
        .nest("/v1/queue_tasks", queue_task::router(queue_svc))
        .nest("/v1/timers", timer::router(timer_svc))
        .nest("/v1/match", match_router::router())
        .route("/v1/healthz", get(|| async { "ok" }))
        .with_state((*state).clone());      // 全局状态
    app
}

use utoipa::OpenApi;
use stepflow_dto::dto;

#[derive(OpenApi)]
#[openapi(
    paths(
        template::create,
        template::list,
        template::get_one,
        template::update,
        template::delete_one,
        execution::start,
        execution::list,
        execution::get_one,
        execution::update,
        execution::delete_one,
        execution::list_by_status,
        worker::poll_task,
        worker::update_task_status,
        activity_task::list_tasks,
        activity_task::get_task,
        activity_task::get_tasks_by_run_id,
        activity_task::start_task,
        activity_task::complete_task,
        activity_task::fail_task,
        activity_task::heartbeat_task,
        workflow_event::list_events,
        workflow_event::get_event,
        workflow_event::record_event,
        workflow_event::archive_event,
        workflow_event::delete_event,
        queue_task::get_one,
        queue_task::update_one,
        queue_task::list_by_status,
        queue_task::validate_dsl,
        queue_task::list_to_retry,
        queue_task::delete_one,
        timer::create_timer,
        timer::get_timer,
        timer::update_timer,
        timer::delete_timer,
        timer::find_before,
        match_router::enqueue_task,
        match_router::poll_task,
        match_router::get_stats,
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
pub struct ApiDoc;
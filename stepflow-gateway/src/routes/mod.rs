use axum::{Router, routing::get};
pub mod template;
pub mod execution;
pub mod worker;
pub mod activity_task;
pub mod workflow_event;
use crate::{
    app_state::AppState, 
    service::{
        template::TemplateSqlxSvc,
        execution::ExecutionSqlxSvc,
        activity_task::ActivityTaskSqlxSvc,
        workflow_event::WorkflowEventSqlxSvc,
    },
};
use std::sync::Arc;

pub fn new(state: AppState) -> Router<AppState> {
    let state = Arc::new(state);
    // 创建要用的 ServiceImpl
    let tpl_svc = TemplateSqlxSvc::new(state.persist.clone());
    let exec_svc = ExecutionSqlxSvc::new(state.clone());
    let event_svc = WorkflowEventSqlxSvc::new(state.persist.clone());
    let task_svc = ActivityTaskSqlxSvc::new(state.persist.clone());
    let app = Router::new()
        .nest("/v1/templates", template::router(tpl_svc))
        .nest("/v1/executions", execution::router(exec_svc))
        .nest("/v1/activity_tasks", activity_task::router(task_svc))
        .nest("/v1/worker", worker::router())
        .nest("/v1/workflow_events", workflow_event::router(event_svc))
        .route("/v1/healthz", get(|| async { "ok" }))
        .with_state((*state).clone());      // 全局状态
    app
}
use axum::{
    extract::{Path, Query, State},
    routing::{get, post, delete},
    Json, Router,
};
use crate::{
    app_state::AppState,
    dto::workflow_event::*,
    error::AppResult,
    service::workflow_event::WorkflowEventService,
};
use utoipa::ToSchema;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_events))
        .route("/", post(record_event))
        .route("/:id", get(get_event))
        .route("/run/:run_id", get(list_events_for_run))
        .route("/:id/archive", post(archive_event))
        .route("/:id", delete(delete_event))
}

/// 获取事件列表
#[utoipa::path(
    get,
    path = "/v1/workflow_events",
    tag = "workflow_events",
    params(ListQuery),
    responses(
        (status = 200, description = "成功获取事件列表", body = Vec<WorkflowEventDto>),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn list_events(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<WorkflowEventDto>>> {
    let events = state.workflow_event_svc.list_events(query.limit, query.offset).await?;
    Ok(Json(events))
}

/// 获取单个事件
#[utoipa::path(
    get,
    path = "/v1/workflow_events/{id}",
    tag = "workflow_events",
    params(
        ("id" = i64, Path, description = "事件ID")
    ),
    responses(
        (status = 200, description = "成功获取事件", body = WorkflowEventDto),
        (status = 404, description = "事件不存在"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<WorkflowEventDto>> {
    let event = state.workflow_event_svc.get_event(id).await?
        .ok_or_else(|| crate::error::Error::NotFound("Event not found".into()))?;
    Ok(Json(event))
}

/// 获取工作流实例的所有事件
#[utoipa::path(
    get,
    path = "/v1/workflow_events/run/{run_id}",
    tag = "workflow_events",
    params(
        ("run_id" = String, Path, description = "工作流实例ID"),
        ListQuery
    ),
    responses(
        (status = 200, description = "成功获取事件列表", body = Vec<WorkflowEventDto>),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn list_events_for_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<WorkflowEventDto>>> {
    let events = state.workflow_event_svc.list_events_for_run(&run_id, query.limit, query.offset).await?;
    Ok(Json(events))
}

/// 记录新事件
#[utoipa::path(
    post,
    path = "/v1/workflow_events",
    tag = "workflow_events",
    request_body = RecordEventRequest,
    responses(
        (status = 200, description = "成功记录事件", body = WorkflowEventDto),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn record_event(
    State(state): State<AppState>,
    Json(req): Json<RecordEventRequest>,
) -> AppResult<Json<WorkflowEventDto>> {
    let event = state.workflow_event_svc.record_event(req).await?;
    Ok(Json(event))
}

/// 归档事件
#[utoipa::path(
    post,
    path = "/v1/workflow_events/{id}/archive",
    tag = "workflow_events",
    params(
        ("id" = i64, Path, description = "事件ID")
    ),
    responses(
        (status = 200, description = "成功归档事件", body = WorkflowEventDto),
        (status = 404, description = "事件不存在"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn archive_event(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<WorkflowEventDto>> {
    let event = state.workflow_event_svc.archive_event(id).await?
        .ok_or_else(|| crate::error::Error::NotFound("Event not found".into()))?;
    Ok(Json(event))
}

/// 删除事件
#[utoipa::path(
    delete,
    path = "/v1/workflow_events/{id}",
    tag = "workflow_events",
    params(
        ("id" = i64, Path, description = "事件ID")
    ),
    responses(
        (status = 200, description = "成功删除事件"),
        (status = 404, description = "事件不存在"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn delete_event(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<()> {
    state.workflow_event_svc.delete_event(id).await?;
    Ok(())
} 
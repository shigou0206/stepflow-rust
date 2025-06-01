use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use crate::{
    app_state::AppState,
    dto::activity_task::*,
    error::AppResult,
    service::ActivityTaskService,
};
use utoipa::{ToSchema, IntoParams};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tasks))
        .route("/:task_token", get(get_task))
        .route("/run/:run_id", get(get_tasks_by_run_id))
        .route("/:task_token/start", post(start_task))
        .route("/:task_token/complete", post(complete_task))
        .route("/:task_token/fail", post(fail_task))
        .route("/:task_token/heartbeat", post(heartbeat_task))
}

/// 获取任务列表
#[utoipa::path(
    get,
    path = "/v1/activity_tasks",
    tag = "activity_tasks",
    params(ListQuery),
    responses(
        (status = 200, description = "成功获取任务列表", body = Vec<ActivityTaskDto>),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<ActivityTaskDto>>> {
    let tasks = state.activity_task_svc.list_tasks(query.limit, query.offset).await?;
    Ok(Json(tasks))
}

/// 获取单个任务
#[utoipa::path(
    get,
    path = "/v1/activity_tasks/{task_token}",
    tag = "activity_tasks",
    params(
        ("task_token" = String, Path, description = "任务标识")
    ),
    responses(
        (status = 200, description = "成功获取任务", body = ActivityTaskDto),
        (status = 404, description = "任务不存在"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn get_task(
    State(state): State<AppState>,
    Path(task_token): Path<String>,
) -> AppResult<Json<ActivityTaskDto>> {
    let task = state.activity_task_svc.get_task(&task_token).await?;
    Ok(Json(task))
}

/// 获取工作流实例的所有任务
#[utoipa::path(
    get,
    path = "/v1/activity_tasks/run/{run_id}",
    tag = "activity_tasks",
    params(
        ("run_id" = String, Path, description = "工作流实例 ID")
    ),
    responses(
        (status = 200, description = "成功获取任务列表", body = Vec<ActivityTaskDto>),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn get_tasks_by_run_id(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> AppResult<Json<Vec<ActivityTaskDto>>> {
    let tasks = state.activity_task_svc.get_tasks_by_run_id(&run_id).await?;
    Ok(Json(tasks))
}

/// 开始执行任务
#[utoipa::path(
    post,
    path = "/v1/activity_tasks/{task_token}/start",
    tag = "activity_tasks",
    params(
        ("task_token" = String, Path, description = "任务标识")
    ),
    responses(
        (status = 200, description = "成功开始任务", body = ActivityTaskDto),
        (status = 404, description = "任务不存在"),
        (status = 400, description = "任务状态错误"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn start_task(
    State(state): State<AppState>,
    Path(task_token): Path<String>,
) -> AppResult<Json<ActivityTaskDto>> {
    let task = state.activity_task_svc.start_task(&task_token).await?;
    Ok(Json(task))
}

/// 完成任务
#[utoipa::path(
    post,
    path = "/v1/activity_tasks/{task_token}/complete",
    tag = "activity_tasks",
    params(
        ("task_token" = String, Path, description = "任务标识")
    ),
    request_body = CompleteRequest,
    responses(
        (status = 200, description = "成功完成任务", body = ActivityTaskDto),
        (status = 404, description = "任务不存在"),
        (status = 400, description = "任务状态错误"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn complete_task(
    State(state): State<AppState>,
    Path(task_token): Path<String>,
    Json(req): Json<CompleteRequest>,
) -> AppResult<Json<ActivityTaskDto>> {
    let task = state.activity_task_svc.complete_task(&task_token, req.result).await?;
    Ok(Json(task))
}

/// 标记任务失败
#[utoipa::path(
    post,
    path = "/v1/activity_tasks/{task_token}/fail",
    tag = "activity_tasks",
    params(
        ("task_token" = String, Path, description = "任务标识")
    ),
    request_body = FailRequest,
    responses(
        (status = 200, description = "成功标记任务失败", body = ActivityTaskDto),
        (status = 404, description = "任务不存在"),
        (status = 400, description = "任务状态错误"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn fail_task(
    State(state): State<AppState>,
    Path(task_token): Path<String>,
    Json(req): Json<FailRequest>,
) -> AppResult<Json<ActivityTaskDto>> {
    let task = state.activity_task_svc.fail_task(&task_token, req.reason, req.details).await?;
    Ok(Json(task))
}

/// 更新任务心跳
#[utoipa::path(
    post,
    path = "/v1/activity_tasks/{task_token}/heartbeat",
    tag = "activity_tasks",
    params(
        ("task_token" = String, Path, description = "任务标识")
    ),
    request_body = HeartbeatRequest,
    responses(
        (status = 200, description = "成功更新心跳", body = ActivityTaskDto),
        (status = 404, description = "任务不存在"),
        (status = 400, description = "任务状态错误"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn heartbeat_task(
    State(state): State<AppState>,
    Path(task_token): Path<String>,
    Json(req): Json<HeartbeatRequest>,
) -> AppResult<Json<ActivityTaskDto>> {
    let task = state.activity_task_svc.heartbeat_task(&task_token, req.details).await?;
    Ok(Json(task))
} 
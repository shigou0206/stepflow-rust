use axum::{
    Router,
    routing::{get, post},
    extract::{Path, Query, State},
    Json,
};
use stepflow_dto::dto::activity_task::{
    ActivityTaskDto, 
    ListQuery, 
    CompleteRequest, 
    FailRequest, 
    HeartbeatRequest};

use crate::{
    app_state::AppState,
    error::AppResult,
    service::{ActivityTaskService, ActivityTaskSvc},
};

pub fn router(svc: ActivityTaskSvc) -> Router<AppState> {
    Router::new()
        .route("/", get(list_tasks))
        .route("/:task_token", get(get_task))
        .route("/by-run/:run_id", get(get_tasks_by_run_id))
        .route("/:task_token/start", post(start_task))
        .route("/:task_token/complete", post(complete_task))
        .route("/:task_token/fail", post(fail_task))
        .route("/:task_token/heartbeat", post(heartbeat_task))
        .with_state(svc)
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
    State(svc): State<ActivityTaskSvc>,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<Vec<ActivityTaskDto>>> {
    Ok(Json(svc.list_tasks(query.limit, query.offset).await?))
}

/// 获取单个任务
#[utoipa::path(
    get,
    path = "/v1/activity_tasks/{task_token}",
    tag = "activity_tasks",
    params(
        ("task_token" = String, Path, description = "任务令牌")
    ),
    responses(
        (status = 200, description = "成功获取任务", body = ActivityTaskDto),
        (status = 404, description = "任务不存在"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn get_task(
    State(svc): State<ActivityTaskSvc>,
    Path(task_token): Path<String>,
) -> AppResult<Json<ActivityTaskDto>> {
    Ok(Json(svc.get_task(&task_token).await?))
}

/// 获取指定运行ID的任务列表
#[utoipa::path(
    get,
    path = "/v1/activity_tasks/by-run/{run_id}",
    tag = "activity_tasks",
    params(
        ("run_id" = String, Path, description = "工作流运行ID")
    ),
    responses(
        (status = 200, description = "成功获取任务列表", body = Vec<ActivityTaskDto>),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn get_tasks_by_run_id(
    State(svc): State<ActivityTaskSvc>,
    Path(run_id): Path<String>,
) -> AppResult<Json<Vec<ActivityTaskDto>>> {
    Ok(Json(svc.get_tasks_by_run_id(&run_id).await?))
}

/// 开始执行任务
#[utoipa::path(
    post,
    path = "/v1/activity_tasks/{task_token}/start",
    tag = "activity_tasks",
    params(
        ("task_token" = String, Path, description = "任务令牌")
    ),
    responses(
        (status = 200, description = "成功开始任务", body = ActivityTaskDto),
        (status = 404, description = "任务不存在"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn start_task(
    State(svc): State<ActivityTaskSvc>,
    Path(task_token): Path<String>,
) -> AppResult<Json<ActivityTaskDto>> {
    Ok(Json(svc.start_task(&task_token).await?))
}

/// 完成任务
#[utoipa::path(
    post,
    path = "/v1/activity_tasks/{task_token}/complete",
    tag = "activity_tasks",
    params(
        ("task_token" = String, Path, description = "任务令牌")
    ),
    request_body = CompleteRequest,
    responses(
        (status = 200, description = "成功完成任务", body = ActivityTaskDto),
        (status = 404, description = "任务不存在"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn complete_task(
    State(svc): State<ActivityTaskSvc>,
    Path(task_token): Path<String>,
    Json(req): Json<CompleteRequest>,
) -> AppResult<Json<ActivityTaskDto>> {
    Ok(Json(svc.complete_task(&task_token, req.result).await?))
}

/// 标记任务失败
#[utoipa::path(
    post,
    path = "/v1/activity_tasks/{task_token}/fail",
    tag = "activity_tasks",
    params(
        ("task_token" = String, Path, description = "任务令牌")
    ),
    request_body = FailRequest,
    responses(
        (status = 200, description = "成功标记任务失败", body = ActivityTaskDto),
        (status = 404, description = "任务不存在"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn fail_task(
    State(svc): State<ActivityTaskSvc>,
    Path(task_token): Path<String>,
    Json(req): Json<FailRequest>,
) -> AppResult<Json<ActivityTaskDto>> {
    Ok(Json(svc.fail_task(&task_token, req).await?))
}

/// 发送任务心跳
#[utoipa::path(
    post,
    path = "/v1/activity_tasks/{task_token}/heartbeat",
    tag = "activity_tasks",
    params(
        ("task_token" = String, Path, description = "任务令牌")
    ),
    request_body = HeartbeatRequest,
    responses(
        (status = 200, description = "成功发送心跳", body = ActivityTaskDto),
        (status = 404, description = "任务不存在"),
        (status = 500, description = "服务器内部错误"),
    )
)]
async fn heartbeat_task(
    State(svc): State<ActivityTaskSvc>,
    Path(task_token): Path<String>,
    Json(req): Json<HeartbeatRequest>,
) -> AppResult<Json<ActivityTaskDto>> {
    Ok(Json(svc.heartbeat_task(&task_token, req).await?))
}

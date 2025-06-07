use axum::{
    routing::get,
    extract::{Path, State, Query},
    Json, Router,
};
use crate::{
    dto::queue_task::{QueueTaskDto, UpdateQueueTaskDto},
    service::{QueueTaskSvc, QueueTaskService},
    error::AppResult,
    app_state::AppState,
};
use std::collections::HashMap;

pub fn router(svc: QueueTaskSvc) -> Router<AppState> {
    Router::new()
        .route("/", get(list_by_status))
        .route("/:id", get(get_one).put(update_one))
        .route("/:id/validate-dsl", get(validate_dsl))
        .with_state(svc)
}

/// 获取任务详情
#[utoipa::path(
    get,
    path = "/v1/queue_tasks/{id}",
    params(
        ("id" = String, Path, description = "任务 ID")
    ),
    responses(
        (status = 200, description = "成功获取任务", body = QueueTaskDto),
        (status = 404, description = "任务不存在"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "queue_tasks"
)]
pub async fn get_one(
    State(svc): State<QueueTaskSvc>,
    Path(id): Path<String>,
) -> AppResult<Json<QueueTaskDto>> {
    Ok(Json(svc.get_task(&id).await?))
}

/// 更新任务
#[utoipa::path(
    put,
    path = "/v1/queue_tasks/{id}",
    params(
        ("id" = String, Path, description = "任务 ID")
    ),
    request_body = UpdateQueueTaskDto,
    responses(
        (status = 200, description = "成功更新任务"),
        (status = 404, description = "任务不存在"),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "queue_tasks"
)]
pub async fn update_one(
    State(svc): State<QueueTaskSvc>,
    Path(id): Path<String>,
    Json(update): Json<UpdateQueueTaskDto>,
) -> AppResult<()> {
    svc.update_task(&id, update).await?;
    Ok(())
}

/// 根据状态列出任务
#[utoipa::path(
    get,
    path = "/v1/queue_tasks",
    params(
        ("status" = String, Query, description = "任务状态"),
        ("limit" = i64, Query, description = "分页大小"),
        ("offset" = i64, Query, description = "分页偏移")
    ),
    responses(
        (status = 200, description = "成功获取任务列表", body = [QueueTaskDto]),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "queue_tasks"
)]
pub async fn list_by_status(
    State(svc): State<QueueTaskSvc>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<Vec<QueueTaskDto>>> {
    let status = params.get("status").cloned().unwrap_or("pending".to_string());
    let limit = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(100);
    let offset = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let list = svc.list_tasks_by_status(&status, limit, offset).await?;
    Ok(Json(list))
}

/// 校验任务的 DSL 格式
#[utoipa::path(
    get,
    path = "/v1/queue_tasks/{id}/validate-dsl",
    params(
        ("id" = String, Path, description = "任务 ID")
    ),
    responses(
        (status = 200, description = "DSL 格式合法"),
        (status = 400, description = "DSL 格式错误"),
        (status = 404, description = "任务不存在"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "queue_tasks"
)]
pub async fn validate_dsl(
    State(svc): State<QueueTaskSvc>,
    Path(id): Path<String>,
) -> AppResult<()> {
    svc.validate_task_dsl(&id).await?;
    Ok(())
}
use std::{collections::HashMap, sync::Arc};
use axum::{
    extract::{Path, Query, State},
    routing::{get, post, delete},
    Json, Router,
};
use chrono::NaiveDateTime;
use stepflow_dto::dto::queue_task::{QueueTaskDto, UpdateQueueTaskDto};
use stepflow_core::{
    app_state::AppState,
    error::{AppError, AppResult},
    service::QueueTaskService,
};

pub fn router(svc: Arc<dyn QueueTaskService>) -> Router<AppState> {
    Router::new()
        .route("/", get(list_by_status))
        .route("/retry", get(list_to_retry))
        .route("/:id", get(get_one).put(update_one).delete(delete_one))
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
    State(svc): State<Arc<dyn QueueTaskService>>,
    Path(id): Path<String>,
) -> AppResult<Json<QueueTaskDto>> {
    Ok(Json(svc.get_task(&id).await?))
}

/// 更新任务
#[utoipa::path(
    put,
    path = "/v1/queue_tasks/{id}",
    request_body = UpdateQueueTaskDto,
    params(
        ("id" = String, Path, description = "任务 ID")
    ),
    responses(
        (status = 200, description = "成功更新任务"),
        (status = 404, description = "任务不存在"),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "queue_tasks"
)]
pub async fn update_one(
    State(svc): State<Arc<dyn QueueTaskService>>,
    Path(id): Path<String>,
    Json(update): Json<UpdateQueueTaskDto>,
) -> AppResult<()> {
    svc.update_task(&id, update).await?;
    Ok(())
}

/// 删除任务
#[utoipa::path(
    delete,
    path = "/v1/queue_tasks/{id}",
    params(
        ("id" = String, Path, description = "任务 ID")
    ),
    responses(
        (status = 200, description = "成功删除任务"),
        (status = 404, description = "任务不存在"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "queue_tasks"
)]
pub async fn delete_one(
    State(svc): State<Arc<dyn QueueTaskService>>,
    Path(id): Path<String>,
) -> AppResult<()> {
    svc.delete_task(&id).await?;
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
    State(svc): State<Arc<dyn QueueTaskService>>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<Vec<QueueTaskDto>>> {
    let status = params.get("status").cloned().unwrap_or_else(|| "pending".to_string());
    let limit = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(100);
    let offset = params.get("offset").and_then(|v| v.parse().ok()).unwrap_or(0);
    let list = svc.list_tasks_by_status(&status, limit, offset).await?;
    Ok(Json(list))
}

/// 查询待重试任务
#[utoipa::path(
    get,
    path = "/v1/queue_tasks/retry",
    params(
        ("before" = String, Query, description = "重试时间上限（ISO 格式）"),
        ("limit" = i64, Query, description = "分页数量")
    ),
    responses(
        (status = 200, description = "成功获取任务列表", body = [QueueTaskDto]),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "queue_tasks"
)]
pub async fn list_to_retry(
    State(svc): State<Arc<dyn QueueTaskService>>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<Vec<QueueTaskDto>>> {
    let before_str = params.get("before").ok_or_else(|| {
        AppError::BadRequest("missing 'before' param".into())
    })?.clone();

    let before = before_str
        .parse::<NaiveDateTime>()
        .map_err(|e| AppError::BadRequest(format!("invalid datetime: {e}")))?;

    let limit = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);
    let list = svc.list_tasks_to_retry(before, limit).await?;
    Ok(Json(list))
}
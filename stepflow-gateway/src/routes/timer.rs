use axum::{
    routing::{get, post},
    extract::{Path, Query, State},
    Json, Router,
};

use stepflow_dto::dto::timer::{TimerDto, CreateTimerDto, UpdateTimerDto};

use crate::{
    app_state::AppState,
    service::{TimerSvc, TimerService},
    error::AppResult,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub fn router(svc: TimerSvc) -> Router<AppState> {
    Router::new()
        .route("/", post(create_timer).get(find_before))
        .route("/:id", get(get_timer).put(update_timer).delete(delete_timer))
        .with_state(svc)
}

/// 创建定时器
#[utoipa::path(
    post,
    path = "/v1/timers",
    request_body = CreateTimerDto,
    responses(
        (status = 200, description = "成功创建定时器", body = TimerDto),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "timers"
)]
pub async fn create_timer(
    State(svc): State<TimerSvc>,
    Json(dto): Json<CreateTimerDto>,
) -> AppResult<Json<TimerDto>> {
    let result = svc.create_timer(dto).await?;
    Ok(Json(result))
}

/// 获取定时器详情
#[utoipa::path(
    get,
    path = "/v1/timers/{id}",
    params(
        ("id" = String, Path, description = "定时器 ID")
    ),
    responses(
        (status = 200, description = "成功获取定时器", body = TimerDto),
        (status = 404, description = "定时器不存在"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "timers"
)]
pub async fn get_timer(
    State(svc): State<TimerSvc>,
    Path(id): Path<String>,
) -> AppResult<Json<TimerDto>> {
    Ok(Json(svc.get_timer(&id).await?))
}

/// 更新定时器
#[utoipa::path(
    put,
    path = "/v1/timers/{id}",
    params(
        ("id" = String, Path, description = "定时器 ID")
    ),
    request_body = UpdateTimerDto,
    responses(
        (status = 200, description = "成功更新定时器", body = TimerDto),
        (status = 400, description = "请求参数错误"),
        (status = 404, description = "定时器不存在"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "timers"
)]
pub async fn update_timer(
    State(svc): State<TimerSvc>,
    Path(id): Path<String>,
    Json(update): Json<UpdateTimerDto>,
) -> AppResult<Json<TimerDto>> {
    Ok(Json(svc.update_timer(&id, update).await?))
}

/// 删除定时器
#[utoipa::path(
    delete,
    path = "/v1/timers/{id}",
    params(
        ("id" = String, Path, description = "定时器 ID")
    ),
    responses(
        (status = 200, description = "成功删除定时器"),
        (status = 404, description = "定时器不存在"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "timers"
)]
pub async fn delete_timer(
    State(svc): State<TimerSvc>,
    Path(id): Path<String>,
) -> AppResult<()> {
    svc.delete_timer(&id).await?;
    Ok(())
}

/// 查询在某个时间点前触发的定时器
#[utoipa::path(
    get,
    path = "/v1/timers/before",
    params(
        ("before" = String, Query, description = "时间点（ISO8601）"),
        ("limit" = i64, Query, description = "返回数量")
    ),
    responses(
        (status = 200, description = "成功获取定时器列表", body = [TimerDto]),
        (status = 400, description = "参数错误"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "timers"
)]
pub async fn find_before(
    State(svc): State<TimerSvc>,
    Query(params): Query<HashMap<String, String>>,
) -> AppResult<Json<Vec<TimerDto>>> {
    let before_str = params.get("before").cloned().ok_or_else(|| {
        crate::error::AppError::BadRequest("缺少 before 参数".into())
    })?;

    let before: DateTime<Utc> = before_str
        .parse()
        .map_err(|e| crate::error::AppError::BadRequest(format!("时间格式错误: {}", e)))?;

    let limit = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(100);

    let timers = svc.find_timers_before(before, limit).await?;
    Ok(Json(timers))
}
use axum::{
    routing::{get, post},
    extract::State,
    Json, Router,
};
use std::time::Duration;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use stepflow_dto::dto::match_stats::*;

/// 添加任务到匹配队列
#[utoipa::path(
    post,
    path = "/v1/match/enqueue",
    request_body = EnqueueRequest,
    tag = "match",
    responses(
        (status = 200, description = "任务已入队"),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn enqueue_task(
    State(app): State<AppState>,
    Json(payload): Json<EnqueueRequest>,
) -> AppResult<()> {
    app.match_service
        .enqueue_task(&payload.queue, payload.task)
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;
    Ok(())
}

/// 拉取任务（worker poll）
#[utoipa::path(
    post,
    path = "/v1/match/poll",
    request_body = PollRequest,
    tag = "match",
    responses(
        (status = 200, description = "返回任务", body = PollResponse),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn poll_task(
    State(app): State<AppState>,
    Json(payload): Json<PollRequest>,
) -> AppResult<Json<PollResponse>> {
    let task = app.match_service
        .poll_task(
            &payload.queue,
            &payload.worker_id,
            Duration::from_secs(payload.timeout_secs.unwrap_or(10)),
        )
        .await;

    Ok(Json(PollResponse {
        has_task: task.is_some(),
        task,
    }))
}

/// 查看队列状态
#[utoipa::path(
    get,
    path = "/v1/match/stats",
    tag = "match",
    responses(
        (status = 200, description = "返回队列统计", body = [MatchStats]),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn get_stats(
    State(app): State<AppState>,
) -> AppResult<Json<Vec<MatchStats>>> {
    let stats = app.match_service.queue_stats().await;
    Ok(Json(stats))
}

/// 组合路由，挂载在 `/v1/match` 下
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/enqueue", post(enqueue_task))
        .route("/poll", post(poll_task))
        .route("/stats", get(get_stats))
}

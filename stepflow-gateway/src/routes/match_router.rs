//! routes/match.rs  —— 适配新版 MatchService

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use std::time::Duration;

use stepflow_core::{
    app_state::AppState,
    error::{AppError, AppResult},
};


use stepflow_dto::dto::{
    match_stats::{EnqueueRequest, MatchStats, PollRequest, PollResponse},
};

/// ① 任务入队
#[utoipa::path(
    post,
    path        = "/v1/match/enqueue",
    request_body = EnqueueRequest,
    tag          = "match",
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

/// ② worker 长轮询取任务  
///    对齐 **take_task(queue, worker_id, timeout)** 接口
#[utoipa::path(
    post,
    path        = "/v1/match/poll",
    request_body = PollRequest,
    tag          = "match",
    responses(
        (status = 200, description = "返回任务", body = PollResponse),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn poll_task(
    State(app): State<AppState>,
    Json(payload): Json<PollRequest>,
) -> AppResult<Json<PollResponse>> {
    let task_opt = app
        .match_service
        .take_task(
            &payload.queue,
            &payload.worker_id,
            Duration::from_secs(payload.timeout_secs.unwrap_or(10)),
        )
        .await;

    Ok(Json(PollResponse {
        has_task: task_opt.is_some(),
        task:     task_opt,
    }))
}

/// ③ 查询队列统计
#[utoipa::path(
    get,
    path = "/v1/match/stats",
    tag  = "match",
    responses(
        (status = 200, description = "返回队列统计", body = [MatchStats]),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn get_stats(State(app): State<AppState>) -> AppResult<Json<Vec<MatchStats>>> {
    Ok(Json(app.match_service.queue_stats().await))
}

/// ④ 组合路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/enqueue", post(enqueue_task))
        .route("/poll",    post(poll_task))
        .route("/stats",   get(get_stats))
}
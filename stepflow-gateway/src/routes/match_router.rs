use axum::{
    routing::{get, post},
    extract::State,
    Json, Router,
};
use std::{sync::Arc, time::Duration};

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use stepflow_dto::dto::match_stats::*;

/// POST /v1/match/enqueue
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

/// POST /v1/match/poll
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

/// GET /v1/match/stats
pub async fn get_stats(
    State(app): State<AppState>,
) -> AppResult<Json<Vec<MatchStats>>> {
    let stats = app.match_service.queue_stats().await;
    Ok(Json(stats))
}

/// 将上面的 handler 组合成一个 `Router<AppState>`
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/enqueue", post(enqueue_task))
        .route("/poll",    post(poll_task))
        .route("/stats",   get(get_stats))
}
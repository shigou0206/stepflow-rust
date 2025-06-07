use axum::{Json, Router, routing::{get, post}};
use std::sync::Arc;
use std::time::Duration;
use axum::extract::State;
use anyhow;

use stepflow_dto::dto::match_stats::*;
use stepflow_match::service::{MatchService, MemoryMatchService};
use crate::error::{AppResult, AppError};

#[utoipa::path(
    post,
    path = "/v1/match/enqueue",
    request_body = EnqueueRequest,
    tag = "match",
    responses(
        (status = 200, description = "任务成功入队"),
        (status = 500, description = "内部错误")
    )
)]
pub async fn enqueue_task(
    State(state): State<MatchServiceState>,
    Json(payload): Json<EnqueueRequest>
) -> AppResult<()> {
    state.hybrid.enqueue_task(&payload.queue, payload.task).await.map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;
    Ok(())
}

#[utoipa::path(
    post,
    path = "/v1/match/poll",
    request_body = PollRequest,
    tag = "match",
    responses(
        (status = 200, description = "获取任务结果", body = PollResponse),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn poll_task(
    State(state): State<MatchServiceState>,
    Json(payload): Json<PollRequest>
) -> AppResult<Json<PollResponse>> {
    let task = state.hybrid.poll_task(
        &payload.queue,
        &payload.worker_id,
        Duration::from_secs(payload.timeout_secs.unwrap_or(10))
    ).await;

    Ok(Json(PollResponse {
        has_task: task.is_some(),
        task,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/match/stats",
    tag = "match",
    responses(
        (status = 200, description = "获取当前队列状态", body = [MatchStats])
    )
)]
pub async fn get_stats(
    State(state): State<MatchServiceState>
) -> AppResult<Json<Vec<MatchStats>>> {
    let memory = state.hybrid
        .as_any()
        .downcast_ref::<MemoryMatchService>()
        .ok_or_else(|| AppError::BadRequest("不是 MemoryMatchService 实例".to_string()))?;

    let mut result = Vec::new();
    for q in ["default_task_queue"] {
        let pending = memory.pending_tasks_count(q).await;
        let waiting = memory.waiting_workers_count(q).await;
        result.push(MatchStats {
            queue: q.to_string(),
            pending_tasks: pending,
            waiting_workers: waiting,
        });
    }

    Ok(Json(result))
}

#[derive(Clone)]
pub struct MatchServiceState {
    pub hybrid: Arc<dyn MatchService>,
}

pub fn router(state: MatchServiceState) -> Router {
    Router::new()
        .route("/enqueue", post(enqueue_task))
        .route("/poll", post(poll_task))
        .route("/stats", get(get_stats))
        .with_state(state)
}
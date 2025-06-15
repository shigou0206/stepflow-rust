//! routes/worker.rs — 适配新 MatchService Trait

use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use chrono::Utc;
use std::time::Duration;
use tracing::{debug, info};    

use stepflow_core::{
    app_state::AppState,
    error::{AppError, AppResult},
};

use stepflow_dto::dto::{
    queue_task::UpdateQueueTaskDto,
    signal::ExecutionSignal,
    worker::{PollRequest, PollResponse, TaskStatus, UpdateRequest},
};

const DEFAULT_QUEUE:  &str = "default_task_queue";
const POLL_TIMEOUT_S: u64  = 30;

// ───────────────────────── poll ────────────────────────────────
#[utoipa::path(
    post,
    path       = "/v1/worker/poll",
    request_body = PollRequest,
    tag        = "worker",
    responses(
        (status = 200, body = PollResponse),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    )
)]
pub async fn poll_task(
    State(app): State<AppState>,
    Json(req): Json<PollRequest>,
) -> AppResult<Json<PollResponse>> {
    info!(
        "[/poll] worker={} queue={} timeout={}s",
        req.worker_id, DEFAULT_QUEUE, POLL_TIMEOUT_S
    );

    let task_opt = app
        .match_service
        .take_task(
            DEFAULT_QUEUE,
            &req.worker_id,
            Duration::from_secs(POLL_TIMEOUT_S),
        )
        .await;

    let resp = match task_opt {
        Some(task) => {
            info!(
                "✅ task found: run_id={} state={}",
                task.run_id, task.state_name
            );
            PollResponse {
                has_task:  true,
                run_id:    Some(task.run_id),
                state_name:Some(task.state_name),
                tool_type: Some(task.resource),
                task_id:   Some(task.task_id),
                input:     task.task_payload,
            }
        }
        None => {
            info!("❌ no task for worker={}", req.worker_id);
            PollResponse::empty()
        }
    };

    Ok(Json(resp))
}

// ───────────────────────── update ──────────────────────────────
#[utoipa::path(
    post,
    path       = "/v1/worker/update",
    request_body = UpdateRequest,
    tag        = "worker",
    responses(
        (status = 200, description = "成功"),
        (status = 404, description = "未找到 workflow"),
        (status = 500, description = "内部错误")
    )
)]
#[axum::debug_handler]
pub async fn update_task_status(
    State(app): State<AppState>,
    Json(req): Json<UpdateRequest>,
) -> AppResult<()> {
    info!(
        run_id=%req.run_id, state=%req.state_name, status=?req.status,
        "📥 worker 更新任务"
    );

    // ── 1️⃣ 同步队列表：completed / failed / cancelled ───────────
    let patch = match req.status {
        TaskStatus::SUCCEEDED => UpdateQueueTaskDto {
            status:        Some("completed".into()),
            task_payload:  Some(Some(req.result.clone())),
            completed_at:  Some(Some(Utc::now())),
            ..Default::default()
        },
        TaskStatus::FAILED => UpdateQueueTaskDto {
            status:        Some("failed".into()),
            error_message: Some(Some(req.result.to_string())),
            failed_at:     Some(Some(Utc::now())),
            ..Default::default()
        },
        TaskStatus::CANCELLED => UpdateQueueTaskDto {
            status:        Some("cancelled".into()),
            error_message: Some(Some(req.result.to_string())),
            failed_at:     Some(Some(Utc::now())),
            ..Default::default()
        },
    };

    app.match_service
        .finish_task(&req.run_id, &req.state_name, patch)
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;

    // ── 2️⃣ 找到内存中的引擎并推送 ExecutionSignal ────────────────
    let mut engines = app.engines.lock().await;
    debug!("🧠 engine count={}", engines.len());

    let engine = engines
        .get_mut(&req.run_id)
        .ok_or(AppError::NotFound)?;

    let signal = match req.status {
        TaskStatus::SUCCEEDED => ExecutionSignal::TaskCompleted {
            run_id:     req.run_id.clone(),
            state_name: req.state_name.clone(),
            output:     req.result.clone(),
        },
        TaskStatus::FAILED => ExecutionSignal::TaskFailed {
            run_id:     req.run_id.clone(),
            state_name: req.state_name.clone(),
            error:      req.result.to_string(),
        },
        TaskStatus::CANCELLED => ExecutionSignal::TaskCancelled {
            run_id:     req.run_id.clone(),
            state_name: req.state_name.clone(),
            reason:     Some(req.result.to_string()),
        },
    };

    engine
        .get_signal_sender()
        .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("no signal sender")))?        
        .send(signal)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("send signal failed: {e}")))?;

    // ── 3️⃣ 处理信号并继续执行 ────────────────────────────────────
    engine
        .handle_next_signal()
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;
    engine
        .advance_until_blocked()
        .await
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!(e)))?;

    // 若执行完毕，移除引擎
    if engine.finished {
        engines.remove(&req.run_id);
        info!(run_id=%req.run_id, "🏁 workflow finished, engine removed");
    }

    Ok(())
}

// ───────────────────────── router ──────────────────────────────
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/poll",   post(poll_task))
        .route("/update", post(update_task_status))
}
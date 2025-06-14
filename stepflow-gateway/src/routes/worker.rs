use axum::{
    extract::State,
    routing::post,
    Json, Router,
};

use stepflow_dto::dto::signal::ExecutionSignal;
use stepflow_dto::dto::worker::{PollRequest, PollResponse, TaskStatus, UpdateRequest};

use crate::{
    app_state::AppState,
    error::{AppError, AppResult},
};

use std::time::Duration;
use tracing::{debug, error, info, warn};

/// --------------------------- /v1/worker/poll ---------------------------
#[utoipa::path(
    post,
    path = "/v1/worker/poll",
    request_body = PollRequest,
    responses(
        (status = 200, description = "成功获取任务", body = PollResponse),
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "worker"
)]
pub async fn poll_task(
    State(state): State<AppState>,
    Json(req): Json<PollRequest>,
) -> AppResult<Json<PollResponse>> {
    const DEFAULT_QUEUE_NAME: &str = "default_task_queue";
    const POLL_TIMEOUT_SECONDS: u64 = 30;

    info!(
        "[/poll] Worker '{}' polling queue '{}' with timeout {}s",
        req.worker_id, DEFAULT_QUEUE_NAME, POLL_TIMEOUT_SECONDS
    );

    match state
        .match_service
        .poll_task(
            DEFAULT_QUEUE_NAME,
            &req.worker_id,
            Duration::from_secs(POLL_TIMEOUT_SECONDS),
        )
        .await
    {
        Some(task) => {
            info!(
                "✅ Task found for worker '{}': run_id: {}, state: {}",
                req.worker_id, task.run_id, task.state_name
            );
            Ok(Json(PollResponse {
                has_task: true,
                run_id: Some(task.run_id),
                state_name: Some(task.state_name),
                input: task.task_payload,
                tool_type: Some(task.resource),
                task_id: Some(task.task_id),
            }))
        }
        None => {
            info!(
                "❌ No task available for worker '{}' in queue '{}'",
                req.worker_id, DEFAULT_QUEUE_NAME
            );
            Ok(Json(PollResponse {
                has_task: false,
                run_id: None,
                state_name: None,
                tool_type: None,
                task_id: None,
                input: None,
            }))
        }
    }
}

/// --------------------------- /v1/worker/update --------------------------
#[utoipa::path(
    post,
    path = "/v1/worker/update",
    request_body = UpdateRequest,
    responses(
        (status = 200, description = "成功更新任务状态"),
        (status = 404, description = "未找到对应工作流实例"),
        (status = 500, description = "服务器内部错误或引擎推进失败")
    ),
    tag = "worker"
)]
#[axum::debug_handler]
pub async fn update_task_status(
    State(state): State<AppState>,
    Json(req): Json<UpdateRequest>,
) -> AppResult<()> {
    info!(
        run_id = %req.run_id,
        state_name = %req.state_name,
        status = ?req.status,
        "📥 Received task update from worker."
    );

    /* ------------------------------------------------------------------
     * ① **补写 queue_tasks 表**：将 processing → completed/failed/cancelled
     * ------------------------------------------------------------------ */
    let persist_result = state
        .match_service
        .update_task_status(
            &req.run_id,
            &req.state_name,
            match req.status {
                TaskStatus::SUCCEEDED => "completed",
                TaskStatus::FAILED    => "failed",
                TaskStatus::CANCELLED => "cancelled",
            },
            &req.result,
        )
        .await;

    if let Err(e) = persist_result {
        // 持久化失败仅告警，不要阻断后续的 signal / 引擎推进
        warn!(
            run_id = %req.run_id,
            task_state = %req.state_name,
            "⚠️ Failed to persist task status: {e}"
        );
    }

    /* ------------------------- ② 后面的逻辑保持不变 ----------------------- */
    let mut engines = state.engines.lock().await;
    debug!("🧠 Current engine count: {}", engines.len());

    let engine = engines
        .get_mut(&req.run_id)
        .ok_or(AppError::NotFound)?;

    info!(
        run_id = %req.run_id,
        current_state = %engine.current_state,
        "✅ Engine found"
    );

    // --- 构造 ExecutionSignal ---
    let signal = match req.status {
        TaskStatus::SUCCEEDED => ExecutionSignal::TaskCompleted {
            run_id: req.run_id.clone(),
            state_name: req.state_name.clone(),
            output: req.result.clone(),
        },
        TaskStatus::FAILED => ExecutionSignal::TaskFailed {
            run_id: req.run_id.clone(),
            state_name: req.state_name.clone(),
            error: req.result.to_string(),
        },
        TaskStatus::CANCELLED => ExecutionSignal::TaskCancelled {
            run_id: req.run_id.clone(),
            state_name: req.state_name.clone(),
            reason: Some(req.result.to_string()),
        },
    };

    // --- 发送 signal ---
    if let Some(sender) = engine.get_signal_sender() {
        sender
            .send(signal)
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("Failed to send signal: {}", e)))?;
        info!(run_id = %req.run_id, "📤 Signal sent to engine");
    } else {
        error!(run_id = %req.run_id, "❌ No signal sender available");
        return Err(AppError::Anyhow(anyhow::anyhow!("No signal sender available")));
    }

    // --- 处理 signal 并推进引擎 ---
    engine.handle_next_signal().await.map_err(|e| {
        error!(run_id = %req.run_id, error = %e, "❌ Failed to handle signal");
        AppError::Anyhow(anyhow::anyhow!("Failed to handle signal: {}", e))
    })?;

    engine.advance_until_blocked().await.map_err(|e| {
        error!(run_id = %req.run_id, error = %e, "❌ Failed to advance engine");
        AppError::Anyhow(anyhow::anyhow!("Failed to advance engine: {}", e))
    })?;

    // --- 结束判断 ---
    if engine.finished {
        info!(run_id = %req.run_id, "🏁 Workflow finished, engine removed");
        engines.remove(&req.run_id);
    } else {
        info!(
            run_id = %req.run_id,
            current_state = %engine.current_state,
            "🔄 Workflow not finished"
        );
    }

    Ok(())
}

/// --------------------------- 路由组合 ----------------------------------
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/poll", post(poll_task))
        .route("/update", post(update_task_status))
}
use axum::{
    routing::post,
    Json, Router,
    extract::State,
};

use stepflow_dto::dto::worker::{PollRequest, PollResponse, UpdateRequest, TaskStatus};
use stepflow_dto::dto::signal::ExecutionSignal;

use crate::{
    error::{AppResult, AppError},
    app_state::AppState,
};
use std::time::Duration;
use tracing::{info, warn, error, debug};

/// Worker 轮询任务
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
                task_id: Some(task.task_id.clone()),
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

/// Worker 更新任务状态
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

    let mut engines = state.engines.lock().await;
    debug!("🧠 Current engine count: {}", engines.len());

    if let Some(engine) = engines.get_mut(&req.run_id) {
        info!(
            run_id = %req.run_id,
            current_state = %engine.current_state,
            "✅ Engine found"
        );

        // Get signal sender from engine
        let signal_sender = match engine.get_signal_sender() {
            Some(sender) => sender,
            None => {
                error!(run_id = %req.run_id, "❌ No signal sender available");
                return Err(AppError::Anyhow(anyhow::anyhow!("No signal sender available")));
            }
        };

        // Create appropriate signal based on task status, using task's state name
        let signal = match req.status {
            TaskStatus::SUCCEEDED => ExecutionSignal::TaskCompleted {
                run_id: req.run_id.clone(),
                state_name: req.state_name.clone(),  // 使用任务状态名
                output: req.result.clone(),
            },
            TaskStatus::FAILED => ExecutionSignal::TaskFailed {
                run_id: req.run_id.clone(),
                state_name: req.state_name.clone(),  // 使用任务状态名
                error: req.result.to_string(),
            },
            TaskStatus::CANCELLED => ExecutionSignal::TaskCancelled {
                run_id: req.run_id.clone(),
                state_name: req.state_name.clone(),  // 使用任务状态名
                reason: Some(req.result.to_string()),
            },
        };

        // Send signal to engine
        if let Err(e) = signal_sender.send(signal) {
            error!(run_id = %req.run_id, error = %e, "❌ Failed to send signal");
            return Err(AppError::Anyhow(anyhow::anyhow!("Failed to send signal: {}", e)));
        }

        info!(run_id = %req.run_id, "📤 Signal sent to engine");

        // 处理信号并推进引擎
        if let Err(e) = engine.handle_next_signal().await {
            error!(run_id = %req.run_id, error = %e, "❌ Failed to handle signal");
            return Err(AppError::Anyhow(anyhow::anyhow!("Failed to handle signal: {}", e)));
        }

        // 推进引擎直到下一个阻塞点
        if let Err(e) = engine.advance_until_blocked().await {
            error!(run_id = %req.run_id, error = %e, "❌ Failed to advance engine");
            return Err(AppError::Anyhow(anyhow::anyhow!("Failed to advance engine: {}", e)));
        }

        // Check if engine is finished after signal processing
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
    } else {
        warn!(run_id = %req.run_id, "❌ Engine not found");
        return Err(AppError::NotFound);
    }

    Ok(())
}

/// Worker 路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/poll", post(poll_task))
        .route("/update", post(update_task_status))
}
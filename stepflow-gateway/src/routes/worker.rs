use axum::{
    routing::post,
    Json, Router,
    extract::State,
};
use crate::{
    dto::worker::{PollRequest, PollResponse, UpdateRequest},
    error::{AppResult, AppError},
    app_state::AppState,
};
use std::time::Duration;

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

    println!(
        "📡 [/poll] Worker '{}' polling queue '{}' with timeout {}s",
        req.worker_id,
        DEFAULT_QUEUE_NAME,
        POLL_TIMEOUT_SECONDS
    );

    match state
        .match_service
        .poll_task(
            DEFAULT_QUEUE_NAME, 
            &req.worker_id, 
            Duration::from_secs(POLL_TIMEOUT_SECONDS)
        )
        .await
    {
        Some(task) => {
            println!("✅ Task found for worker '{}': run_id: {}, state: {}", 
                req.worker_id, task.run_id, task.state_name);
            Ok(Json(PollResponse {
                has_task: true,
                run_id: Some(task.run_id),
                state_name: Some(task.state_name),
                input: task.input,
            }))
        }
        None => {
            println!("❌ No task available for worker '{}' in queue '{}' within timeout", 
                req.worker_id, DEFAULT_QUEUE_NAME);
            Ok(Json(PollResponse {
                has_task: false,
                run_id: None,
                state_name: None,
                input: None,
            }))
        }
    }
}

/// Worker 路由
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/poll", post(poll_task))
        .route("/update", post(update))
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
pub async fn update(
    State(state): State<AppState>,
    Json(req): Json<UpdateRequest>,
) -> AppResult<()> {
    tracing::info!(
        run_id = %req.run_id, 
        state_name = %req.state_name, 
        status = %req.status, 
        "📥 [/update] Received task status update from worker."
    );
    
    let mut engines = state.engines.lock().await;
    tracing::debug!("🧠 Current engine count: {}", engines.len());

    if let Some(engine) = engines.get_mut(&req.run_id) {
        tracing::info!(
            run_id = %req.run_id, 
            current_engine_state = %engine.current_state, 
            "✅ Engine found."
        );

        engine.context = req.result.clone();
        tracing::debug!(run_id = %req.run_id, context = ?engine.context, "📦 Engine context updated.");

        match engine.advance_until_blocked().await {
            Ok(_) => {
                tracing::info!(run_id = %req.run_id, "🎯 Engine advanced successfully (or already blocked/finished).");
            }
            Err(e) => {
                tracing::error!(run_id = %req.run_id, error = %e, "❌ Engine advancement failed.");
                return Err(AppError::Anyhow(anyhow::anyhow!(
                    "Engine advancement failed for run_id {}: {}", req.run_id, e
                )));
            }
        }

        if engine.finished {
            tracing::info!(run_id = %req.run_id, "🏁 Workflow finished. Removing engine from memory.");
            engines.remove(&req.run_id);
        } else {
            tracing::info!(run_id = %req.run_id, current_engine_state = %engine.current_state, "🔄 Workflow not yet finished, remains in memory.");
        }
    } else {
        tracing::warn!(run_id = %req.run_id, "❌ Engine not found for task update.");
        return Err(AppError::NotFound);
    }

    Ok(())
}
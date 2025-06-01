use axum::{
    routing::post,
    Json, Router,
    extract::State,
};
use crate::{
    dto::worker::{PollRequest, PollResponse, UpdateRequest},
    error::AppResult,
    app_state::AppState,
};
use stepflow_engine::engine::WorkflowMode;

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
    let engines = state.engines.lock().await;
    
    // 找到一个 Deferred 模式的引擎
    for (run_id, engine) in engines.iter() {
        if engine.mode == WorkflowMode::Deferred {
            // TODO: 检查是否有可执行的任务
            return Ok(Json(PollResponse {
                has_task: true,
                run_id: Some(run_id.clone()),
                state_name: Some(engine.current_state.clone()),
                input: Some(engine.context.clone()),
            }));
        }
    }
    
    Ok(Json(PollResponse {
        has_task: false,
        run_id: None,
        state_name: None,
        input: None,
    }))
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
        (status = 400, description = "请求参数错误"),
        (status = 500, description = "服务器内部错误")
    ),
    tag = "worker"
)]
pub async fn update(
    State(state): State<AppState>,
    Json(req): Json<UpdateRequest>,
) -> AppResult<()> {
    let mut engines = state.engines.lock().await;
    if let Some(engine) = engines.get_mut(&req.run_id) {
        // 更新引擎状态
        engine.context = req.result;
        
        // 推进引擎
        if let Err(e) = engine.advance_once().await {
            tracing::error!("引擎推进失败: {}", e);
            return Ok(());
        }
        
        // 如果工作流完成，从 Map 中移除
        if engine.finished {
            engines.remove(&req.run_id);
        }
    }
    Ok(())
}

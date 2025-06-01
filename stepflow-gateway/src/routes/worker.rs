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

    println!("📡 [/poll] Worker 请求任务: worker_id = {}", req.worker_id);
    println!("🧠 当前内存中引擎数量: {}", engines.len());

    for (run_id, engine) in engines.iter() {
        println!("🔍 检查引擎: {}, 当前状态: {}, 模式: {:?}", run_id, engine.current_state, engine.mode);

        if engine.mode == WorkflowMode::Deferred {
            println!("✅ 分配任务: run_id = {}, state = {}", run_id, engine.current_state);

            return Ok(Json(PollResponse {
                has_task: true,
                run_id: Some(run_id.clone()),
                state_name: Some(engine.current_state.clone()),
                input: Some(engine.context.clone()),
            }));
        }
    }

    println!("❌ 当前无可执行任务（Deferred）");
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
    println!("📥 [/update] 收到状态上报: run_id = {}, state_name = {}, status = {}",
        req.run_id, req.state_name, req.status);
    
    let mut engines = state.engines.lock().await;
    println!("🧠 当前引擎数量: {}", engines.len());

    if let Some(engine) = engines.get_mut(&req.run_id) {
        println!("✅ 找到引擎: 当前状态 = {}", engine.current_state);

        engine.context = req.result.clone();
        println!("📦 更新上下文: {:?}", engine.context);

        match engine.advance_until_blocked().await {
            Ok(_) => println!("🎯 引擎推进完成（或已阻塞/完成）"),
            Err(e) => println!("❌ 引擎推进失败: {}", e),
        }

        if engine.finished {
            println!("✅ 工作流已完成，准备移除引擎 {}", req.run_id);
            engines.remove(&req.run_id);
        }
    } else {
        println!("❌ 没有找到对应引擎: run_id = {}", req.run_id);
    }

    Ok(())
}
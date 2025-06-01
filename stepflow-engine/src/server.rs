// stepflow-engine/src/server.rs

use warp::{Filter, Rejection, Reply};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use sqlx::SqlitePool;

use stepflow_dsl::dsl::WorkflowDSL;
use stepflow_engine::engine::{
    WorkflowEngine, WorkflowMode,
    memory_stub::{MemoryStore, MemoryQueue},
};
use stepflow_engine::http::poll::{poll_route, JsonError, handle_poll, PollRequest}; // ✅ 引入 handle_poll 和 PollRequest
use stepflow_storage::PersistenceManagerImpl;
use stepflow_hook::{EngineEventDispatcher, impls::{log_hook::LogHook, persist_hook::PersistHook, ws_hook::WsHook}};
use tokio::sync::mpsc::unbounded_channel;
use stepflow_hook::ui_event::UiEvent;
use log::{debug, info, error, warn};
use env_logger;

/// 所有 Deferred 引擎实例都存放在这个全局 Map 里
/// Key = run_id，Value = 对应的 WorkflowEngine<MemoryStore, MemoryQueue>
type Engines = Arc<Mutex<HashMap<String, WorkflowEngine<MemoryStore, MemoryQueue>>>>;

/// 启动时收到的 /start 请求
#[derive(Debug, Deserialize)]
struct StartRequest {
    pub run_id: String,
    pub dsl: Value,
    pub init_ctx: Option<Value>,
}

#[derive(Debug, Serialize)]
struct StartResponse {
    pub success: bool,
    pub message: String,
}

/// Worker 调度后上报的 /update 请求
#[derive(Debug, Deserialize)]
struct UpdateRequest {
    pub run_id: String,
    pub state_name: String,
    pub status: String,
    pub result: Value,
}

#[derive(Debug, Serialize)]
struct UpdateResponse {
    pub success: bool,
    pub context: Value,
    pub message: String,
}

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();
    
    info!("🚀 StepFlow Engine 启动");

    // 1. 全局共享一个 Map，保存所有 Deferred 模式下的引擎
    let engines: Engines = Arc::new(Mutex::new(HashMap::new()));
    
    // Create SQLite pool
    let pool = SqlitePool::connect("/Users/sryu/projects/stepflow-rust/stepflow-sqlite/data/data.db")
        .await
        .expect("Failed to create SQLite pool");

    // Create WebSocket channel for UI events
    let (ws_tx, _ws_rx) = unbounded_channel::<UiEvent>();
    let ws_tx_for_hook = ws_tx.clone();

    // Create persistence manager
    let persistence_manager = Arc::new(PersistenceManagerImpl::new(pool.clone()));
    let persistence_manager_for_hook = persistence_manager.clone();
    let persistence_manager_filter = warp::any().map(move || persistence_manager.clone());

    // Create event dispatcher with LogHook, PersistHook and WsHook
    let event_dispatcher = Arc::new(EngineEventDispatcher::new(vec![
        LogHook::new(),
        PersistHook::new(persistence_manager_for_hook),
        WsHook::new(ws_tx_for_hook),
    ]));
    let event_dispatcher_filter = warp::any().map(move || event_dispatcher.clone());

    // Define filters for shared resources
    let engines_filter = warp::any().map(move || engines.clone());
    let pool_filter = warp::any().map(move || pool.clone());
    
    // Welcome route
    let welcome = warp::get()
        .and(warp::path::end())
        .map(|| warp::reply::html("Welcome to StepFlow Engine"));

    // Poll route
    let poll = warp::path("poll")
        .and(warp::post())
        .and(warp::body::json())
        .and(engines_filter.clone())
        .and_then(handle_poll);

    // Start route
    let start = warp::path("start")
        .and(warp::post())
        .and(warp::body::json())
        .and(engines_filter.clone())
        .and(pool_filter.clone())
        .and(persistence_manager_filter.clone())
        .and(event_dispatcher_filter.clone())
        .and_then(start_handler);

    // Update route
    let update = warp::path("update")
        .and(warp::post())
        .and(warp::body::json())
        .and(engines_filter.clone())
        .and_then(update_handler);

    // 组合所有路由
    let routes = welcome
        .or(poll)
        .or(start)
        .or(update)  // 添加 update 路由
        .with(warp::log("stepflow_engine"))
        .recover(|err: Rejection| async move {
            error!("请求处理错误: {:?}", err);
            if err.is_not_found() {
                Ok(warp::reply::with_status(
                    "Route not found",
                    warp::http::StatusCode::NOT_FOUND,
                ))
            } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
                Ok(warp::reply::with_status(
                    "Invalid JSON format",
                    warp::http::StatusCode::BAD_REQUEST,
                ))
            } else {
                Ok::<_, std::convert::Infallible>(warp::reply::with_status(
                    "Internal server error",
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        });

    // 启动服务器
    let port = 3031;
    info!("📡 服务器监听 http://127.0.0.1:{}", port);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

/// 处理 "POST /start"
async fn start_handler(
    req: StartRequest, 
    engines: Engines, 
    pool: SqlitePool,
    persistence_manager: Arc<PersistenceManagerImpl>,
    event_dispatcher: Arc<EngineEventDispatcher>,
) -> Result<impl Reply, Rejection> {
    info!("📥 收到启动请求: run_id = {}", req.run_id);
    
    // 1. 把 JSON Value 反序列化为 WorkflowDSL
    let dsl: WorkflowDSL = match serde_json::from_value(req.dsl.clone()) {
        Ok(x) => x,
        Err(e) => {
            error!("❌ DSL 解析失败: {}", e);
            let resp = StartResponse {
                success: false,
                message: format!("Failed to parse DSL: {}", e),
            };
            return Ok(warp::reply::json(&resp));
        }
    };
    info!("✅ DSL 解析成功");

    // 2. 拿到 init_ctx，默认为空对象
    let init_ctx = req.init_ctx.clone().unwrap_or_else(|| Value::Object(Default::default()));
    info!("📦 初始 context: {:?}", init_ctx);

    // 3. 构造一个 Deferred 模式的 WorkflowEngine
    let mut engine = WorkflowEngine::new(
        req.run_id.clone(),
        dsl,
        init_ctx,
        WorkflowMode::Deferred,  // 确保是 Deferred 模式
        MemoryStore::new(persistence_manager),
        MemoryQueue::new(),
        pool.clone(),
        event_dispatcher,
    );
    info!("🚀 引擎实例创建完成");

    // 4. 把引擎加入 Map
    {
        let mut map = engines.lock().await;
        info!("当前活跃引擎数: {}", map.len());
        map.insert(req.run_id.clone(), engine);
        info!("✅ 引擎 {} 启动完成并加入 Map", req.run_id);
    }

    // // 5. 获取引擎并执行一次
    // if let Some(engine) = engines.lock().await.get_mut(&req.run_id) {
    //     if let Err(e) = engine.advance_once().await {
    //         error!("❌ Engine.advance_once() 失败: {}", e);
    //         let resp = StartResponse {
    //             success: false,
    //             message: format!("Engine.advance_once() failed: {}", e),
    //         };
    //         return Ok(warp::reply::json(&resp));
    //     }
    //     info!("✅ 首次 advance_once 成功");
    // }
    // 5. 获取引擎并执行多步推进
    if let Some(engine) = engines.lock().await.get_mut(&req.run_id) {
        if let Err(e) = engine.advance_until_blocked().await {
            error!("❌ Engine.advance_until_blocked() 失败: {}", e);
            let resp = StartResponse {
                success: false,
                message: format!("Engine.advance_until_blocked() failed: {}", e),
            };
            return Ok(warp::reply::json(&resp));
        }
        info!("✅ 引擎已推进直到阻塞或完成");
    }

    let resp = StartResponse {
        success: true,
        message: "Workflow started in Deferred mode".into(),
    };
    Ok(warp::reply::json(&resp))
}

/// 处理 "POST /update"
async fn update_handler(req: UpdateRequest, engines: Engines) -> Result<impl Reply, Rejection> {
    info!("📥 收到更新请求: run_id = {}, state = {}, status = {}", 
          req.run_id, req.state_name, req.status);
    
    let mut map = engines.lock().await;
    info!("🔒 获取引擎锁, 当前活跃引擎数: {}", map.len());
    
    // 1. 找到对应的 engine
    if let Some(engine) = map.get_mut(&req.run_id) {
        info!("✅ 找到引擎 {}", req.run_id);
        
        // 2. 把 Worker 上传回来的结果直接覆盖 engine.context
        engine.context = req.result.clone();
        info!("📦 更新 context: {:?}", engine.context);

        // 3. 推进引擎：循环调用 advance_once，直到碰到下一个 Deferred Task (should_continue=false) 或 执行结束
        let mut should_remove = false;
        loop {
            match engine.advance_once().await {
                Ok(step) => {
                    if !step.should_continue {
                        should_remove = engine.finished;
                        info!("🔄 引擎推进完成: finished = {}", engine.finished);
                        break;
                    }
                    info!("🔄 继续推进引擎...");
                }
                Err(e) => {
                    error!("❌ Engine.advance_once() 失败: {}", e);
                    let resp = UpdateResponse {
                        success: false,
                        context: engine.context.clone(),
                        message: format!("Engine.advance_once() failed: {}", e),
                    };
                    return Ok(warp::reply::json(&resp));
                }
            }
        }

        let context = engine.context.clone();
        let message = "Workflow resumed and updated".into();

        // 4. 如果引擎已完成，从 Map 中移除
        if should_remove {
            map.remove(&req.run_id);
            info!("🗑️ 引擎 {} 已完成，从 Map 中移除", req.run_id);
        }

        let resp = UpdateResponse {
            success: true,
            context,
            message,
        };
        Ok(warp::reply::json(&resp))
    } else {
        error!("❌ 未找到引擎: {}", req.run_id);
        let resp = UpdateResponse {
            success: false,
            context: Value::Null,
            message: format!("No workflow found for run_id = {}", req.run_id),
        };
        Ok(warp::reply::json(&resp))
    }
}
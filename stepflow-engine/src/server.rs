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
use stepflow_engine::http::poll::poll_route; // ✅ 引入我们上面写好的 poll_route
use stepflow_storage::PersistenceManagerImpl;

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
    // Create SQLite pool
    let pool = SqlitePool::connect("sqlite:database.sqlite")
        .await
        .expect("Failed to create SQLite pool");

    // Create persistence manager
    let persistence_manager = Arc::new(PersistenceManagerImpl::new(pool.clone()));
    let persistence_manager_filter = warp::any().map(move || persistence_manager.clone());

    // 1. 全局共享一个 Map，保存所有 Deferred 模式下的引擎
    let engines: Engines = Arc::new(Mutex::new(HashMap::new()));
    let engines_for_filter = engines.clone();
    let engines_filter = warp::any().map(move || engines_for_filter.clone());
    let pool_filter = warp::any().map(move || pool.clone());

    // ========== 定义 /start ==========
    let start_route = warp::path("start")
        .and(warp::post())
        .and(warp::body::json())
        .and(engines_filter.clone())
        .and(pool_filter.clone())
        .and(persistence_manager_filter.clone())
        .and_then(start_handler);

    // ========== 定义 /update ==========
    let update_route = warp::path("update")
        .and(warp::post())
        .and(warp::body::json())
        .and(engines_filter.clone())
        .and_then(update_handler);

    // ========== 定义 /poll ==========
    // 注意：这里直接把 Arc<Mutex<HashMap<...>>> 传入 poll_route
    let poll_route = poll_route(engines.clone());

    // 合并所有路由
    let routes = start_route.or(update_route).or(poll_route);

    println!("🚀 Starting WorkflowEngine HTTP server on 0.0.0.0:3030 …");
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

/// 处理 "POST /start"
async fn start_handler(
    req: StartRequest, 
    engines: Engines, 
    pool: SqlitePool,
    persistence_manager: Arc<PersistenceManagerImpl>,
) -> Result<impl Reply, Rejection> {
    // 1. 把 JSON Value 反序列化为 WorkflowDSL
    let dsl: WorkflowDSL = match serde_json::from_value(req.dsl.clone()) {
        Ok(x) => x,
        Err(e) => {
            let resp = StartResponse {
                success: false,
                message: format!("Failed to parse DSL: {}", e),
            };
            return Ok(warp::reply::json(&resp));
        }
    };

    // 2. 拿到 init_ctx，默认为空对象
    let init_ctx = req.init_ctx.clone().unwrap_or_else(|| Value::Object(Default::default()));

    // 3. 构造一个 Deferred 模式的 WorkflowEngine
    //    注意传入 MemoryStore、MemoryQueue::new()
    let mut engine = WorkflowEngine::new(
        req.run_id.clone(),
        dsl,
        init_ctx,
        WorkflowMode::Deferred,
        MemoryStore::new(persistence_manager),
        MemoryQueue::new(),
        pool.clone(),
    );

    // 4. "先执行一次 advance_once" —— 这会让第一个 Task 写入到内存队列
    if let Err(e) = engine.advance_once().await {
        let resp = StartResponse {
            success: false,
            message: format!("Engine.advance_once() failed: {}", e),
        };
        return Ok(warp::reply::json(&resp));
    }

    // 5. 把整个 engine 实例存到全局 Map 里
    engines.lock().await.insert(req.run_id.clone(), engine);

    let resp = StartResponse {
        success: true,
        message: "Workflow started in Deferred mode".into(),
    };
    Ok(warp::reply::json(&resp))
}

/// 处理 "POST /update"
async fn update_handler(req: UpdateRequest, engines: Engines) -> Result<impl Reply, Rejection> {
    let mut map = engines.lock().await;
    // 1. 找到对应的 engine
    if let Some(engine) = map.get_mut(&req.run_id) {
        // 2. 把 Worker 上传回来的结果直接覆盖 engine.context
        engine.context = req.result.clone();

        // 3. 推进引擎：循环调用 advance_once，直到碰到下一个 Deferred Task (should_continue=false) 或 执行结束
        loop {
            match engine.advance_once().await {
                Ok(step) => {
                    if !step.should_continue {
                        // should_continue==false：要么挂起到另一个 Task，要么整个流程结束
                        break;
                    }
                    // should_continue=true：说明接下来都是 Pass/Choice/Succeed 等，可以继续推进
                }
                Err(e) => {
                    let resp = UpdateResponse {
                        success: false,
                        context: engine.context.clone(),
                        message: format!("Engine.advance_once() failed: {}", e),
                    };
                    return Ok(warp::reply::json(&resp));
                }
            }
        }

        let resp = UpdateResponse {
            success: true,
            context: engine.context.clone(),
            message: "Workflow resumed and updated".into(),
        };
        Ok(warp::reply::json(&resp))
    } else {
        // run_id 不存在
        let resp = UpdateResponse {
            success: false,
            context: Value::Null,
            message: format!("No workflow found for run_id = {}", req.run_id),
        };
        Ok(warp::reply::json(&resp))
    }
}
// stepflow-engine/src/http/poll.rs

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Rejection, Reply};

use crate::engine::{WorkflowEngine, memory_stub::{MemoryQueue, MemoryStore}};

/// 来自 HTTP 请求的结构：Worker 在轮询时会携带自己的 ID（可选）
#[derive(Debug, Deserialize)]
pub struct PollRequest {
    pub worker_id: String,
}

/// 给 Worker 返回的结构：是否有任务？是哪条任务？输入是什么？
#[derive(Debug, Serialize)]
pub struct PollResponse {
    pub has_task: bool,
    pub run_id: Option<String>,
    pub state_name: Option<String>,
    pub input: Option<Value>,
}

#[derive(Debug)]
pub struct JsonError();

impl warp::reject::Reject for JsonError {}

/// poll_route 接口：
///   - 先把 `engines` 拷贝到闭包里
///   - 每次收到 POST /poll 时，从所有引擎的 MemoryQueue 中尝试 pop 一条任务
///   - 如果找到，返回对应 `run_id`、`state_name` 和该引擎的当前 `context.clone()`
///   - 如果所有引擎都空，返回 `has_task: false`
pub fn poll_route(
    engines: Arc<Mutex<HashMap<String, WorkflowEngine<MemoryStore, MemoryQueue>>>>,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    use log::info;
    use warp::Filter;
    
    info!("👋 注册 /poll 路由");
    
    let json_body = warp::body::json::<PollRequest>();
    let engines = warp::any().map(move || {
        info!("🔄 克隆 engines 引用");
        engines.clone()
    });
    
    warp::path!("poll")
        .and(warp::post())
        .and(json_body)
        .and(engines)
        .and_then(handle_poll)
        .boxed()
}

pub async fn handle_poll(
    req: PollRequest,
    engines: Arc<Mutex<HashMap<String, WorkflowEngine<MemoryStore, MemoryQueue>>>>,
) -> Result<impl Reply, Rejection> {
    use log::info;
    
    info!("📥 收到 poll 请求: worker_id = {}", req.worker_id);

    // 1. 锁住所有引擎
    let mut map = engines.lock().await;
    info!("🔒 获取引擎锁, 当前活跃引擎数: {}", map.len());

    // 2. 遍历所有 run_id 对应的 WorkflowEngine，尝试从它的 MemoryQueue pop 一条任务
    for (run_id, engine) in map.iter_mut() {
        info!("🔍 检查引擎 {}", run_id);
        // 直接访问 engine.queue 内部的 VecDeque< (String, String) >
        let mut guard = engine.queue.0.lock().await;
        info!("  - 队列长度: {}", guard.len());
        
        if let Some((r, state_name)) = guard.pop_front() {
            // 找到一条任务：r 应该等于 run_id（因为 push 时是同一个 run_id），但我们仍然按 r 发回
            // input 就是引擎此时的 context.clone()
            let input = engine.context.clone();
            info!("✅ 找到任务: {} @ {}", r, state_name);
            info!("  - context: {:?}", input);
            let resp = PollResponse {
                has_task: true,
                run_id: Some(r),
                state_name: Some(state_name),
                input: Some(input),
            };
            return Ok(warp::reply::json(&resp));
        }
        info!("  - 队列为空，继续检查下一个引擎");
    }

    // 3. 如果所有队列都空，就告诉 Worker 暂时没有任务
    info!("📭 所有引擎队列都为空");
    let resp = PollResponse {
        has_task: false,
        run_id: None,
        state_name: None,
        input: None,
    };
    Ok(warp::reply::json(&resp))
}

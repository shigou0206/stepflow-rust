// stepflow-engine/src/worker.rs

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
use log::info;

// 使用 reqwest 作为 HTTP 客户端
use reqwest::Client;

#[derive(Debug, Deserialize)]
struct PollResponse {
    has_task: bool,
    run_id: Option<String>,
    state_name: Option<String>,
    input: Option<Value>,
}

#[derive(Debug, Serialize)]
struct UpdateRequest {
    run_id: String,
    state_name: String,
    status: String,
    result: Value,
}

#[tokio::main]
async fn main() {
    // 在 Worker 里也打开一个简单的日志系统
    env_logger::init();
    let client = Client::new();
    let server_base = "http://127.0.0.1:3031";
    let worker_id = "worker-001";

    info!("Worker 启动, server_base = {}, worker_id = {}", server_base, worker_id);

    loop {
        // 1. 拉取任务
        let poll_req = serde_json::json!({ "worker_id": worker_id });
        let resp = client
            .post(&format!("{}/poll", server_base))
            .json(&poll_req)
            .send()
            .await
            .unwrap();

        let poll_body: PollResponse = resp.json().await.unwrap();
        if poll_body.has_task {
            // 拿到一条任务
            let run_id = poll_body.run_id.clone().unwrap();
            let state_name = poll_body.state_name.clone().unwrap();
            let input = poll_body.input.clone().unwrap_or_else(|| serde_json::json!({}));

            info!("Worker 拿到任务 → run_id = {}, state_name = {}, input = {}", run_id, state_name, input);

            // 2. 模拟“执行 echo 工具”：简单把 input 里面加一个 "_ran" 键
            let mut result = input.clone();
            result["_ran"] = Value::String(format!("tool::echo"));

            // 3. 上报执行结果到 /update
            let update_req = UpdateRequest {
                run_id: run_id.clone(),
                state_name: state_name.clone(),
                status: "SUCCEEDED".into(),
                result: result.clone(),
            };

            let update_resp = client
                .post(&format!("{}/update", server_base))
                .json(&update_req)
                .send()
                .await
                .unwrap();

            let update_body: serde_json::Value = update_resp.json().await.unwrap();
            info!("Worker 调用 /update 得到：{}", update_body);
        } else {
            // 没有任务就休眠几秒，再继续 polling
            info!("Worker: 暂时没有任务，2 秒后再拉取...");
            sleep(Duration::from_secs(2)).await;
        }
    }
}
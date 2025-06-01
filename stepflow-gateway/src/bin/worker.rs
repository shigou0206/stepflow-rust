use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct WorkerCfg {
    pub worker_id: String,
    pub server: String,  // e.g. "http://127.0.0.1:3030"
}

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
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // 配置
    let cfg = WorkerCfg {
        worker_id: "worker-1".into(),
        server: "http://127.0.0.1:3000".into(),
    };

    // 启动工作循环
    worker_loop(cfg).await
}

/// 后台轮询协程
async fn worker_loop(cfg: WorkerCfg) -> Result<()> {
    let client = Client::new();
    let mut backoff = 1u64;

    info!(worker = %cfg.worker_id, "worker loop started");

    loop {
        match poll_once(&client, &cfg).await {
            Ok(Some(task)) => {
                backoff = 1;
                if let Err(e) = handle_task(&client, &cfg, task).await {
                    warn!("task failed: {e:?}");
                }
            }
            Ok(None) => {
                debug!("no task, sleep {backoff}s");
                sleep(Duration::from_secs(backoff)).await;
                backoff = (backoff * 2).min(30);
            }
            Err(e) => {
                warn!("poll error: {e:?}, sleep {backoff}s");
                sleep(Duration::from_secs(backoff)).await;
                backoff = (backoff * 2).min(30);
            }
        }
    }
}

/// /poll → Option<Task>
async fn poll_once(client: &Client, cfg: &WorkerCfg)
    -> Result<Option<(String, String, Value)>>
{
    let resp = client
        .post(format!("{}/v1/worker/poll", cfg.server))
        .json(&json!({ "worker_id": cfg.worker_id }))
        .send()
        .await?
        .error_for_status()?
        .json::<PollResponse>()
        .await?;

    if resp.has_task {
        Ok(Some((
            resp.run_id.unwrap(),
            resp.state_name.unwrap(),
            resp.input.unwrap_or_default(),
        )))
    } else {
        Ok(None)
    }
}

/// 业务处理 (示例 Echo)
async fn handle_task(
    client: &Client,
    cfg: &WorkerCfg,
    (run_id, state, mut ctx): (String, String, Value),
) -> Result<()> {
    info!(%run_id, %state, "execute task");

    ctx["_ran"] = Value::String("tool::echo".into());

    client
        .post(format!("{}/v1/worker/update", cfg.server))
        .json(&UpdateRequest {
            run_id,
            state_name: state,
            status: "SUCCEEDED".into(),
            result: ctx,
        })
        .send()
        .await?
        .error_for_status()?;

    Ok(())
} 
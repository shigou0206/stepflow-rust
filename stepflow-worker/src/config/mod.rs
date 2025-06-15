use anyhow::{Result, anyhow};
use std::env;

/// Worker 运行模式（由 WORKER_MODE 控制）
/// - Polling     => 传统长轮询
/// - EventDriven => 基于事件总线（EventBus）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerMode {
    Polling,
    EventDriven,
}

/// 每个 Worker 实例的配置（由环境变量控制）
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub worker_id: String,              // WORKER_ID + index
    pub gateway_server_url: String,     // GATEWAY_SERVER_URL
    pub capabilities: Vec<String>,      // WORKER_CAPABILITIES
    pub mode: WorkerMode,               // WORKER_MODE
}

impl WorkerConfig {
    /// 从环境变量构造 WorkerConfig（支持多实例 index）
    pub fn from_env(index: usize) -> Result<Self> {
        // —— 基础 Worker ID ——（支持多实例自动编号）
        let base_id = env::var("WORKER_ID").unwrap_or_else(|_| "tool-worker".to_string());
        let worker_id = format!("{}-{}", base_id, index);

        // —— 后端地址 ——（默认为本地服务）
        let gateway_server_url = env::var("GATEWAY_SERVER_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:3000/v1/worker".to_string());

        // —— 模式：poll / event ——（默认 polling）
        let mode_str = env::var("WORKER_MODE").unwrap_or_else(|_| "poll".to_string());
        let mode = match mode_str.to_lowercase().as_str() {
            "poll" | "polling" => WorkerMode::Polling,
            "event" | "eventdriven" => WorkerMode::EventDriven,
            other => {
                return Err(anyhow!(
                    "Unsupported WORKER_MODE: {}. Use 'poll' or 'event'",
                    other
                ));
            }
        };

        // —— 能力声明 ——（默认支持所有内建工具）
        let capabilities = env::var("WORKER_CAPABILITIES")
            .unwrap_or_else(|_| "http,shell,file".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        Ok(Self {
            worker_id,
            gateway_server_url,
            capabilities,
            mode,
        })
    }
}
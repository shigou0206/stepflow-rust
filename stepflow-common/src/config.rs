use anyhow::{anyhow, Result};
use std::{env, fmt};

/// 系统运行模式（统一控制 Engine / Worker / MatchService 行为）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepflowMode {
    Polling,
    EventDriven,
}

impl StepflowMode {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "poll" | "polling" => Ok(Self::Polling),
            "event" | "eventdriven" => Ok(Self::EventDriven),
            other => Err(anyhow!("Unsupported mode: {} (use 'poll' or 'event')", other)),
        }
    }
}

impl fmt::Display for StepflowMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepflowMode::Polling => write!(f, "polling"),
            StepflowMode::EventDriven => write!(f, "event-driven"),
        }
    }
}

/// 通用配置结构：适用于 Engine / Worker / Gateway
#[derive(Debug, Clone)]
pub struct StepflowConfig {
    pub mode: StepflowMode,
    pub db_path: String,
    pub worker_id: String,
    pub gateway_server_url: String,
    pub capabilities: Vec<String>,
    pub gateway_bind: String,
    pub concurrency: usize,
}

impl StepflowConfig {
    /// 用于构建第 `index` 个 Worker 配置
    pub fn from_env(index: usize) -> Result<Self> {
        let mode = StepflowMode::from_str(
            &env::var("STEPFLOW_MODE").unwrap_or_else(|_| "event".into())
        )?;

        let db_path = env::var("STEPFLOW_DB_PATH")
            .unwrap_or_else(|_| "data/stepflow.db".into())
            .trim()
            .to_string();

        let base_id = env::var("WORKER_ID").unwrap_or_else(|_| "tool-worker".into());
        let worker_id = format!("{}-{}", base_id.trim(), index);

        let gateway_server_url = env::var("GATEWAY_SERVER_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:3000/v1/worker".into())
            .trim()
            .to_string();

        let capabilities = env::var("WORKER_CAPABILITIES")
            .unwrap_or_else(|_| "http,shell,file".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let gateway_bind = env::var("GATEWAY_BIND")
            .unwrap_or_else(|_| "127.0.0.1:3000".into())
            .trim()
            .to_string();

        let concurrency = env::var("WORKER_CONCURRENCY")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|&c| c > 0)
            .unwrap_or(4);

        Ok(Self {
            mode,
            db_path,
            worker_id,
            gateway_server_url,
            capabilities,
            gateway_bind,
            concurrency,
        })
    }

    /// 单实例调用，index = 0
    pub fn from_env_default() -> Result<Self> {
        Self::from_env(0)
    }

    /// 根据 WORKER_COUNT 构建多个 Worker 配置
    pub fn from_env_all() -> Result<Vec<Self>> {
        let count = env::var("WORKER_COUNT")
            .unwrap_or_else(|_| "1".into())
            .parse::<usize>()
            .map_err(|e| anyhow!("Invalid WORKER_COUNT: {}", e))?;

        (0..count)
            .map(Self::from_env)
            .collect::<Result<Vec<_>>>()
    }

    /// 判断当前 worker 是否支持某个 resource 能力
    pub fn supports(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// 打印配置概要（用于日志）
    pub fn summary(&self) -> String {
        format!(
            "mode={}, worker_id={}, db_path={}, concurrency={}, capabilities={:?}",
            self.mode, self.worker_id, self.db_path, self.concurrency, self.capabilities
        )
    }
}
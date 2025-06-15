use anyhow::{anyhow, Result};
use std::env;

/// 系统运行模式（统一控制 Engine / Worker / MatchService 行为）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepflowMode {
    Polling,
    EventDriven,
}

impl StepflowMode {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "poll" | "polling" => Ok(Self::Polling),
            "event" | "eventdriven" => Ok(Self::EventDriven),
            other => Err(anyhow!("Unsupported mode: {} (use 'poll' or 'event')", other)),
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
}

impl StepflowConfig {
    /// 用于构建第 `index` 个 Worker 配置
    pub fn from_env(index: usize) -> Result<Self> {
        let mode = StepflowMode::from_str(
            &env::var("STEPFLOW_MODE").unwrap_or_else(|_| "event".to_string())
        )?;

        let db_path = env::var("STEPFLOW_DB_PATH").unwrap_or_else(|_| "data/stepflow.db".into());

        let base_id = env::var("WORKER_ID").unwrap_or_else(|_| "tool-worker".into());
        let worker_id = format!("{}-{}", base_id, index);

        let gateway_server_url = env::var("GATEWAY_SERVER_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:3000/v1/worker".into());

        let capabilities = env::var("WORKER_CAPABILITIES")
            .unwrap_or_else(|_| "http,shell,file".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            mode,
            db_path,
            worker_id,
            gateway_server_url,
            capabilities,
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
            .map(|i| Self::from_env(i))
            .collect::<Result<Vec<_>>>()
    }
}
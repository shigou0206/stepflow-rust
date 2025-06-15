use anyhow::{anyhow, Result};
use std::env;

/// 系统运行模式（统一控制 Engine/Worker/MatchService 行为）
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

/// 通用配置结构：适用于 Engine、Worker、Gateway
#[derive(Debug, Clone)]
pub struct StepflowConfig {
    pub mode: StepflowMode,
    pub db_path: String,
    pub worker_id: String,
    pub gateway_server_url: String,
    pub capabilities: Vec<String>,
}

impl StepflowConfig {
    pub fn from_env(index: usize) -> Result<Self> {
        let mode = StepflowMode::from_str(
            &env::var("STEPFLOW_MODE").unwrap_or_else(|_| "poll".to_string())
        )?;

        let db_path = env::var("STEPFLOW_DB_PATH").unwrap_or_else(|_| "data/stepflow.db".into());
        let base_id = env::var("WORKER_ID").unwrap_or_else(|_| "tool-worker".into());
        let worker_id = format!("{}-{}", base_id, index);

        let gateway_server_url = env::var("GATEWAY_SERVER_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:3000/v1/worker".into());

        let capabilities = env::var("WORKER_CAPABILITIES")
            .unwrap_or_else(|_| "http,shell,file".to_string())
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
}
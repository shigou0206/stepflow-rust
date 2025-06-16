use anyhow::{anyhow, Result};
use std::{env, fmt};

/// 系统运行时环境（控制入口行为）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepflowRuntime {
    Gateway,
    FrbOnly,
}

impl StepflowRuntime {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "gateway" => Ok(Self::Gateway),
            "frb" | "frbonly" => Ok(Self::FrbOnly),
            other => Err(anyhow!("Unsupported runtime: {} (use 'gateway' or 'frb')", other)),
        }
    }
}

impl fmt::Display for StepflowRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepflowRuntime::Gateway => write!(f, "gateway"),
            StepflowRuntime::FrbOnly => write!(f, "frb-only"),
        }
    }
}

/// 系统执行模式（控制状态推进机制）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepflowExecMode {
    Polling,
    EventDriven,
}

impl StepflowExecMode {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "poll" | "polling" => Ok(Self::Polling),
            "event" | "eventdriven" => Ok(Self::EventDriven),
            other => Err(anyhow!("Unsupported exec mode: {} (use 'poll' or 'event')", other)),
        }
    }
}

impl fmt::Display for StepflowExecMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepflowExecMode::Polling => write!(f, "polling"),
            StepflowExecMode::EventDriven => write!(f, "event-driven"),
        }
    }
}

/// 通用配置结构：适用于 Engine / Worker / Gateway / FRB
#[derive(Debug, Clone)]
pub struct StepflowConfig {
    pub runtime: StepflowRuntime,
    pub exec_mode: StepflowExecMode,
    pub db_path: String,
    pub worker_id: String,
    pub gateway_server_url: String,
    pub capabilities: Vec<String>,
    pub gateway_bind: String,
    pub concurrency: usize,
}

impl StepflowConfig {
    /// 构建单个配置（根据 index 拼接 worker_id）
    pub fn from_env(index: usize) -> Result<Self> {
        let runtime = StepflowRuntime::from_str(
            &env::var("STEPFLOW_RUNTIME").unwrap_or_else(|_| "gateway".into())
        )?;

        let exec_mode = StepflowExecMode::from_str(
            &env::var("STEPFLOW_EXEC_MODE").unwrap_or_else(|_| "event".into())
        )?;

        // ❗ 校验：FRB 模式只能搭配 EventDriven
        if runtime == StepflowRuntime::FrbOnly && exec_mode != StepflowExecMode::EventDriven {
            return Err(anyhow!("❌ FrbOnly runtime requires EventDriven execution mode"));
        }

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
            runtime,
            exec_mode,
            db_path,
            worker_id,
            gateway_server_url,
            capabilities,
            gateway_bind,
            concurrency,
        })
    }

    /// 构建默认 worker 配置（index = 0）
    pub fn from_env_default() -> Result<Self> {
        Self::from_env(0)
    }

    /// 构建多个 worker 配置（用于并发启动）
    pub fn from_env_all() -> Result<Vec<Self>> {
        let count = env::var("WORKER_COUNT")
            .unwrap_or_else(|_| "1".into())
            .parse::<usize>()
            .map_err(|e| anyhow!("Invalid WORKER_COUNT: {}", e))?;

        (0..count).map(Self::from_env).collect()
    }

    /// 当前 worker 是否支持某个 resource 能力
    pub fn supports(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// 日志摘要
    pub fn summary(&self) -> String {
        format!(
            "runtime={}, exec_mode={}, worker_id={}, db_path={}, concurrency={}, capabilities={:?}",
            self.runtime, self.exec_mode, self.worker_id, self.db_path, self.concurrency, self.capabilities
        )
    }

    pub fn for_flutter() -> Result<Self> {
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow!("无法获取当前可执行文件目录"))?;

        let db_path = exe_dir.join("stepflow.db").to_string_lossy().to_string();

        Ok(Self {
            runtime: StepflowRuntime::FrbOnly,
            exec_mode: StepflowExecMode::EventDriven,
            db_path,
            worker_id: "frb-worker".to_string(),
            gateway_server_url: "".into(),
            capabilities: vec!["http".into(), "shell".into()],
            gateway_bind: "127.0.0.1:3000".into(),
            concurrency: 2,
        })
    }
}
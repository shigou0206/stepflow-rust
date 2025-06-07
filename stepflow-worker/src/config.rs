use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub worker_id: String,
    pub gateway_server_url: String,
}

impl WorkerConfig {
    pub fn from_env(index: usize) -> Result<Self> {
        let base_id = env::var("WORKER_ID").unwrap_or_else(|_| "tool-worker".to_string());
        let gateway_server_url = env::var("GATEWAY_SERVER_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:3000/v1/worker".to_string());
        let worker_id = format!("{}-{}", base_id, index);
        Ok(Self {
            worker_id,
            gateway_server_url,
        })
    }
}
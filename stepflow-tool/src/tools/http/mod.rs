use anyhow::Context;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;
use reqwest::{Client, Method};
use tracing::debug;

use crate::core::tool::{Tool, ToolMetadata};
use crate::common::config::ToolConfig;
use crate::common::context::ToolContext;
use crate::common::result::{ToolResult, ToolMetadata as ResultMetadata};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub timeout: Option<u64>,
    pub headers: Option<HashMap<String, String>>,
    pub verify_ssl: bool,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout: Some(30),
            headers: None,
            verify_ssl: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpInput {
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Value>,
    pub query: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpOutput {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Value,
    pub duration: u64,
}

pub struct HttpTool {
    client: Client,
    _config: HttpConfig,
}

impl HttpTool {
    pub fn new(config: Option<HttpConfig>) -> Self {
        let cfg = config.unwrap_or_default();
        let client = Client::builder()
            .timeout(Duration::from_secs(cfg.timeout.unwrap_or(30)))
            .build()
            .expect("Failed to create HTTP client");
        Self { client, _config: cfg }
    }
}

#[async_trait]
impl Tool for HttpTool {
    fn kind(&self) -> &'static str {
        "http"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "HTTP Tool".to_string(),
            description: "HTTP request tool for making API calls".to_string(),
            version: "1.0.0".to_string(),
            author: "StepFlow".to_string(),
            tags: vec!["http".to_string(), "api".to_string()],
        }
    }

    fn default_config(&self) -> ToolConfig {
        ToolConfig::default()
    }

    fn validate_input(&self, input: &Value, _context: &ToolContext) -> anyhow::Result<()> {
        // 直接把扁平化的 input 当成 HttpInput 来校验
        serde_json::from_value::<HttpInput>(input.clone())
            .context("Invalid HTTP tool input: expected fields url/method")?;
        Ok(())
    }

    async fn execute(&self, input: Value, context: ToolContext) -> anyhow::Result<ToolResult> {
        // 直接把扁平化的 input 当成 HttpInput 来解析
        let http_input: HttpInput = serde_json::from_value(input)
            .context("Invalid HTTP tool input: expected fields url/method")?;
        debug!(?http_input, "HTTP Tool got flattened input");

        let start = std::time::Instant::now();
        let method = Method::from_bytes(http_input.method.to_uppercase().as_bytes())?;
        let mut req = self.client.request(method, &http_input.url);

        if let Some(q) = http_input.query {
            req = req.query(&q);
        }
        if let Some(hdrs) = http_input.headers {
            for (k, v) in hdrs {
                req = req.header(&k, v);
            }
        }
        if let Some(body) = http_input.body {
            req = req.json(&body);
        }

        // 发送请求并取回原始 bytes
        let resp = req.send().await?;
        let status = resp.status().as_u16();
        let headers = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
            .collect::<HashMap<_, _>>();

        let bytes = resp.bytes().await?;
        // 尝试 JSON 解析，失败时回退为 String
        let body = match serde_json::from_slice::<Value>(&bytes) {
            Ok(json) => json,
            Err(_) => Value::String(String::from_utf8_lossy(&bytes).to_string()),
        };

        let duration = start.elapsed().as_millis() as u64;
        let output = HttpOutput { status, headers, body, duration };

        let meta = ResultMetadata {
            duration: context.duration(),
            attempts: context.attempt,
            resource_usage: json!({
                "request_duration_ms": duration,
                "status_code": status,
            }),
            extra: Value::Null,
        };

        Ok(ToolResult::new(json!(output), meta))
    }
}
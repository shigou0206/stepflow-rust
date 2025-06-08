use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::time::Duration;
use reqwest::{Client, Method};

use crate::core::tool::{Tool, ToolMetadata};
use crate::common::config::ToolConfig;
use crate::common::context::ToolContext;
use crate::common::result::{ToolResult, ToolMetadata as ResultMetadata};

use stepflow_dto::dto::tool::ToolInputPayload;

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
    config: HttpConfig,
}

impl HttpTool {
    pub fn new(config: Option<HttpConfig>) -> Self {
        let config = config.unwrap_or_default();
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout.unwrap_or(30)))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
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
        let payload: ToolInputPayload = serde_json::from_value(input.clone())?;
        let _: HttpInput = serde_json::from_value(payload.parameters)?;
        Ok(())
    }

    async fn execute(&self, input: Value, context: ToolContext) -> anyhow::Result<ToolResult> {
        let payload: ToolInputPayload = serde_json::from_value(input)?;
        let http_input: HttpInput = serde_json::from_value(payload.parameters)?;

        let start_time = std::time::Instant::now();

        // 构建请求
        let method = Method::from_bytes(http_input.method.to_uppercase().as_bytes())?;
        let mut request = self.client.request(method, &http_input.url);

        if let Some(query) = http_input.query {
            request = request.query(&query);
        }

        if let Some(headers) = http_input.headers {
            for (key, value) in headers {
                request = request.header(&key, value);
            }
        }

        if let Some(body) = http_input.body {
            request = request.json(&body);
        }

        let response = request.send().await?;
        let status = response.status();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = response.json::<Value>().await?;
        let duration = start_time.elapsed().as_millis() as u64;

        let output = HttpOutput {
            status: status.as_u16(),
            headers,
            body,
            duration,
        };

        let metadata = ResultMetadata {
            duration: context.duration(),
            attempts: context.attempt,
            resource_usage: json!({
                "request_duration_ms": duration,
                "status_code": status.as_u16(),
            }),
            extra: Value::Null,
        };

        Ok(ToolResult::new(json!(output), metadata))
    }
}
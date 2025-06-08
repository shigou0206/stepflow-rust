use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;

use crate::core::tool::{Tool, ToolMetadata};
use crate::common::config::ToolConfig;
use crate::common::context::ToolContext;
use crate::common::result::{ToolResult, ToolMetadata as ResultMetadata};
use stepflow_dto::dto::tool::ToolInputPayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    pub timeout: Option<u64>,
    pub working_dir: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub shell: Option<String>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            timeout: Some(30),
            working_dir: None,
            env: None,
            shell: Some("/bin/sh".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellInput {
    pub command: String,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub working_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: u64,
}

pub struct ShellTool {
    config: ShellConfig,
}

impl ShellTool {
    pub fn new(config: Option<ShellConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }
}

#[async_trait]
impl Tool for ShellTool {
    fn kind(&self) -> &'static str {
        "shell"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "Shell Tool".to_string(),
            description: "Execute shell commands".to_string(),
            version: "1.0.0".to_string(),
            author: "StepFlow".to_string(),
            tags: vec!["shell".to_string(), "command".to_string()],
        }
    }

    fn default_config(&self) -> ToolConfig {
        ToolConfig::default()
    }

    fn validate_input(&self, input: &Value, _context: &ToolContext) -> anyhow::Result<()> {
        let payload: ToolInputPayload = serde_json::from_value(input.clone())?;
        let _: ShellInput = serde_json::from_value(payload.parameters)?; // ✅ Validate only parameters
        Ok(())
    }

    async fn execute(&self, input: Value, context: ToolContext) -> anyhow::Result<ToolResult> {
        let payload: ToolInputPayload = serde_json::from_value(input)?;
        let shell_input: ShellInput = serde_json::from_value(payload.parameters)?;
        let _logical_input = payload.input; // ✅ input is optional logical context, not used here

        let start_time = std::time::Instant::now();

        let mut command = if let Some(shell) = &self.config.shell {
            let mut cmd = Command::new(shell);
            cmd.arg("-c").arg(&shell_input.command);
            cmd
        } else {
            let mut cmd = Command::new(&shell_input.command);
            if let Some(args) = &shell_input.args {
                cmd.args(args);
            }
            cmd
        };

        if let Some(working_dir) = shell_input.working_dir.or_else(|| self.config.working_dir.clone()) {
            command.current_dir(working_dir);
        }

        if let Some(env) = shell_input.env.or_else(|| self.config.env.clone()) {
            for (key, value) in env {
                command.env(key, value);
            }
        }

        let output = command.output().await?;
        let duration = start_time.elapsed().as_millis() as u64;

        let shell_output = ShellOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            duration,
        };

        let metadata = ResultMetadata {
            duration: context.duration(),
            attempts: context.attempt,
            resource_usage: json!({
                "execution_duration_ms": duration,
                "exit_code": shell_output.exit_code,
            }),
            extra: Value::Null,
        };

        Ok(ToolResult::new(json!(shell_output), metadata))
    }
}
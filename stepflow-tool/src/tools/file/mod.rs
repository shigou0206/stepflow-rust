use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::core::tool::{Tool, ToolMetadata}; // ✅ 引入 ToolInputPayload
use crate::common::config::ToolConfig;
use crate::common::context::ToolContext;
use crate::common::result::{ToolResult, ToolMetadata as ResultMetadata};
use stepflow_dto::dto::tool::ToolInputPayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    pub base_path: Option<PathBuf>,
    pub create_dirs: bool,
    pub overwrite: bool,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            base_path: None,
            create_dirs: true,
            overwrite: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInput {
    pub path: PathBuf,
    pub operation: FileOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FileOperation {
    Read,
    Write { content: String },
    Delete,
    List { pattern: Option<String> },
    Copy { target: PathBuf },
    Move { target: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOutput {
    pub success: bool,
    pub path: PathBuf,
    pub content: Option<String>,
    pub files: Option<Vec<PathBuf>>,
    pub error: Option<String>,
}

pub struct FileTool {
    config: FileConfig,
}

impl FileTool {
    pub fn new(config: Option<FileConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    fn resolve_path(&self, path: &PathBuf) -> PathBuf {
        if let Some(base) = &self.config.base_path {
            base.join(path)
        } else {
            path.clone()
        }
    }
}

#[async_trait]
impl Tool for FileTool {
    fn kind(&self) -> &'static str {
        "file"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "File Tool".to_string(),
            description: "File system operations".to_string(),
            version: "1.0.0".to_string(),
            author: "StepFlow".to_string(),
            tags: vec!["file".to_string(), "fs".to_string()],
        }
    }

    fn default_config(&self) -> ToolConfig {
        ToolConfig::default()
    }

    fn validate_input(&self, input: &Value, _context: &ToolContext) -> anyhow::Result<()> {
        let payload: ToolInputPayload = serde_json::from_value(input.clone())?; // ✅ 解包 payload
        let _: FileInput = serde_json::from_value(payload.parameters)?;         // ✅ 校验参数格式
        Ok(())
    }

    async fn execute(&self, input: Value, context: ToolContext) -> anyhow::Result<ToolResult> {
        let payload: ToolInputPayload = serde_json::from_value(input)?;         // ✅ 解包 payload
        let file_input: FileInput = serde_json::from_value(payload.parameters)?; // ✅ 提取 FileInput
        let _logical_input = payload.input; // 目前 FileTool 不用 input，可用于 future 拓展

        let path = self.resolve_path(&file_input.path);
        let start_time = std::time::Instant::now();

        let result = match file_input.operation {
            FileOperation::Read => {
                let mut file = fs::File::open(&path).await?;
                let mut content = String::new();
                file.read_to_string(&mut content).await?;
                FileOutput {
                    success: true,
                    path: path.clone(),
                    content: Some(content),
                    files: None,
                    error: None,
                }
            }
            FileOperation::Write { content } => {
                if let Some(parent) = path.parent() {
                    if self.config.create_dirs {
                        fs::create_dir_all(parent).await?;
                    }
                }
                let mut file = fs::File::create(&path).await?;
                file.write_all(content.as_bytes()).await?;
                FileOutput {
                    success: true,
                    path: path.clone(),
                    content: None,
                    files: None,
                    error: None,
                }
            }
            FileOperation::Delete => {
                fs::remove_file(&path).await?;
                FileOutput {
                    success: true,
                    path: path.clone(),
                    content: None,
                    files: None,
                    error: None,
                }
            }
            FileOperation::List { pattern } => {
                let mut files = Vec::new();
                let mut entries = fs::read_dir(&path).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let p = entry.path();
                    if let Some(pattern) = &pattern {
                        if !p.to_string_lossy().contains(pattern) {
                            continue;
                        }
                    }
                    files.push(p);
                }
                FileOutput {
                    success: true,
                    path: path.clone(),
                    content: None,
                    files: Some(files),
                    error: None,
                }
            }
            FileOperation::Copy { target } => {
                let target = self.resolve_path(&target);
                fs::copy(&path, &target).await?;
                FileOutput {
                    success: true,
                    path: target,
                    content: None,
                    files: None,
                    error: None,
                }
            }
            FileOperation::Move { target } => {
                let target = self.resolve_path(&target);
                fs::rename(&path, &target).await?;
                FileOutput {
                    success: true,
                    path: target,
                    content: None,
                    files: None,
                    error: None,
                }
            }
        };

        let duration = start_time.elapsed().as_millis() as u64;

        let metadata = ResultMetadata {
            duration: context.duration(),
            attempts: context.attempt,
            resource_usage: json!({
                "operation_duration_ms": duration,
                "success": result.success,
            }),
            extra: Value::Null,
        };

        Ok(ToolResult::new(json!(result), metadata))
    }
}
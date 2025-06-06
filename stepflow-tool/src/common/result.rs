use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// 工具执行的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// 执行时长
    pub duration: Duration,
    
    /// 重试次数
    pub attempts: u32,
    
    /// 资源使用情况
    pub resource_usage: Value,
    
    /// 其他元数据
    pub extra: Value,
}

/// 工具执行的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// 输出数据
    pub output: Value,
    
    /// 元数据
    pub metadata: ToolMetadata,
    
    /// 执行日志
    pub logs: Vec<String>,
}

impl ToolResult {
    /// 创建新的执行结果
    pub fn new(output: Value, metadata: ToolMetadata) -> Self {
        Self {
            output,
            metadata,
            logs: Vec::new(),
        }
    }

    /// 添加日志
    pub fn add_log(&mut self, log: String) {
        self.logs.push(log);
    }

    /// 设置输出
    pub fn with_output(mut self, output: Value) -> Self {
        self.output = output;
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, metadata: ToolMetadata) -> Self {
        self.metadata = metadata;
        self
    }
} 
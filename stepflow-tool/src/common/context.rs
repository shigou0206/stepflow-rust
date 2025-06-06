use std::time::{Duration, Instant};
use serde_json::Value;

use crate::common::config::ToolConfig;

/// 工具执行的上下文
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// 执行ID
    pub execution_id: String,
    
    /// 状态名称
    pub state_name: String,
    
    /// 当前重试次数
    pub attempt: u32,
    
    /// 工具配置
    pub config: ToolConfig,
    
    /// 开始时间
    pub start_time: Instant,
    
    /// 额外数据
    pub extra: Value,
}

impl Default for ToolContext {
    fn default() -> Self {
        Self {
            execution_id: String::new(),
            state_name: String::new(),
            attempt: 1,
            config: ToolConfig::default(),
            start_time: Instant::now(),
            extra: Value::Null,
        }
    }
}

impl ToolContext {
    /// 创建新的执行上下文
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取执行时长
    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// 设置额外数据
    pub fn with_extra(mut self, extra: Value) -> Self {
        self.extra = extra;
        self
    }

    /// 增加重试次数
    pub fn increment_attempt(&mut self) {
        self.attempt += 1;
    }
} 
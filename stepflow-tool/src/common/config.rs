use serde::{Deserialize, Serialize};
use std::time::Duration;

use stepflow_dto::dto::error_policy::RetryPolicy;
use crate::core::tool::Validation;

/// 工具执行的配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// 执行超时时间
    pub timeout: Option<Duration>,
    
    /// 重试策略
    pub retry: Option<RetryPolicy>,
    
    /// 输入输出验证规则
    pub validation: Option<Validation>,
    
    /// 最大并发执行数
    pub max_concurrent_executions: Option<usize>,
    
    /// 是否启用日志
    pub enable_logging: bool,
    
    /// 是否启用监控
    pub enable_monitoring: bool,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)),
            retry: None,
            validation: None,
            max_concurrent_executions: Some(10),
            enable_logging: true,
            enable_monitoring: true,
        }
    }
} 
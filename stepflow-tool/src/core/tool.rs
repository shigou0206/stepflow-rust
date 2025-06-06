use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

use crate::common::config::ToolConfig;
use crate::common::context::ToolContext;
use crate::common::result::ToolResult;

/// 重试策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl RetryPolicy {
    pub fn new(max_attempts: u32, initial_delay: Duration, max_delay: Duration, backoff_factor: f64) -> Self {
        Self {
            max_attempts,
            initial_delay,
            max_delay,
            backoff_factor,
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
        }
    }

    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return self.initial_delay;
        }
        let delay = self.initial_delay.as_secs_f64() * self.backoff_factor.powi(attempt as i32);
        Duration::from_secs_f64(delay.min(self.max_delay.as_secs_f64()))
    }
}

/// 输入输出验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    pub input_schema: Option<Value>,
    pub output_schema: Option<Value>,
}

impl Validation {
    pub fn new(input_schema: Option<Value>, output_schema: Option<Value>) -> Self {
        Self {
            input_schema,
            output_schema,
        }
    }

    pub fn empty() -> Self {
        Self {
            input_schema: None,
            output_schema: None,
        }
    }

    pub fn has_validation(&self) -> bool {
        self.input_schema.is_some() || self.output_schema.is_some()
    }
}

/// 工具元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub tags: Vec<String>,
}

/// 工具接口
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具类型标识
    fn kind(&self) -> &'static str;
    
    /// 获取工具元数据
    fn metadata(&self) -> ToolMetadata;
    
    /// 获取工具默认配置
    fn default_config(&self) -> ToolConfig;
    
    /// 验证输入
    fn validate_input(&self, input: &Value, context: &ToolContext) -> anyhow::Result<()>;
    
    /// 执行工具
    async fn execute(&self, input: Value, context: ToolContext) -> anyhow::Result<ToolResult>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use serde_json::json;
    use async_trait::async_trait;
    use crate::common::config::ToolConfig;
    use crate::common::context::ToolContext;
    use crate::common::result::{ToolResult, ToolMetadata as ResultMetadata};

    #[test]
    fn test_retry_policy_defaults() {
        let policy = RetryPolicy::with_defaults();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.initial_delay, Duration::from_secs(1));
        assert_eq!(policy.max_delay, Duration::from_secs(30));
        assert_eq!(policy.backoff_factor, 2.0);
    }

    #[test]
    fn test_retry_policy_delay_calculation() {
        let policy = RetryPolicy::with_defaults();
        
        // 测试初始延迟
        assert_eq!(policy.calculate_delay(0), Duration::from_secs(1));
        
        // 测试指数退避
        assert_eq!(policy.calculate_delay(1), Duration::from_secs(2));
        assert_eq!(policy.calculate_delay(2), Duration::from_secs(4));
        assert_eq!(policy.calculate_delay(3), Duration::from_secs(8));
        
        // 测试最大延迟限制
        assert_eq!(policy.calculate_delay(10), Duration::from_secs(30));
    }

    #[test]
    fn test_retry_policy_custom() {
        let policy = RetryPolicy::new(
            5,
            Duration::from_millis(100),
            Duration::from_secs(1),
            3.0,
        );
        
        assert_eq!(policy.max_attempts, 5);
        assert_eq!(policy.initial_delay, Duration::from_millis(100));
        assert_eq!(policy.max_delay, Duration::from_secs(1));
        assert_eq!(policy.backoff_factor, 3.0);
    }

    #[test]
    fn test_validation_empty() {
        let validation = Validation::empty();
        assert!(validation.input_schema.is_none());
        assert!(validation.output_schema.is_none());
        assert!(!validation.has_validation());
    }

    #[test]
    fn test_validation_with_schemas() {
        let input_schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            }
        });
        
        let output_schema = json!({
            "type": "object",
            "properties": {
                "id": { "type": "number" }
            }
        });

        let validation = Validation::new(Some(input_schema.clone()), Some(output_schema.clone()));
        
        assert!(validation.has_validation());
        assert_eq!(validation.input_schema.as_ref().unwrap(), &input_schema);
        assert_eq!(validation.output_schema.as_ref().unwrap(), &output_schema);
    }

    #[test]
    fn test_validation_partial_schema() {
        let input_schema = json!({
            "type": "string"
        });

        let validation = Validation::new(Some(input_schema.clone()), None);
        assert!(validation.input_schema.is_some());
        assert!(validation.output_schema.is_none());
        assert!(validation.has_validation());
    }

    struct MockTool;

    #[async_trait]
    impl Tool for MockTool {
        fn kind(&self) -> &'static str {
            "mock"
        }

        fn metadata(&self) -> ToolMetadata {
            ToolMetadata {
                name: "Mock Tool".to_string(),
                description: "A mock tool for testing".to_string(),
                version: "1.0.0".to_string(),
                author: "Test".to_string(),
                tags: vec!["test".to_string(), "mock".to_string()],
            }
        }

        fn default_config(&self) -> ToolConfig {
            ToolConfig::default()
        }

        fn validate_input(&self, input: &Value, _context: &ToolContext) -> anyhow::Result<()> {
            if input.get("valid").and_then(Value::as_bool).unwrap_or(false) {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Invalid input"))
            }
        }

        async fn execute(&self, input: Value, context: ToolContext) -> anyhow::Result<ToolResult> {
            Ok(ToolResult::new(
                json!({ "success": true, "input": input }),
                ResultMetadata {
                    duration: context.duration(),
                    attempts: context.attempt,
                    resource_usage: json!({}),
                    extra: Value::Null,
                },
            ))
        }
    }

    #[tokio::test]
    async fn test_mock_tool() {
        let tool = MockTool;
        
        // 测试基本属性
        assert_eq!(tool.kind(), "mock");
        assert_eq!(tool.metadata().name, "Mock Tool");
        
        // 测试输入验证
        let valid_input = json!({ "valid": true });
        let invalid_input = json!({ "valid": false });
        
        assert!(tool.validate_input(&valid_input, &ToolContext::default()).is_ok());
        assert!(tool.validate_input(&invalid_input, &ToolContext::default()).is_err());
        
        // 测试执行
        let context = ToolContext::default();
        let result = tool.execute(valid_input.clone(), context).await.unwrap();
        
        let output: Value = result.output;
        assert_eq!(output.get("success").unwrap(), &json!(true));
        assert_eq!(output.get("input").unwrap(), &valid_input);
    }
}
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::common::config::ToolConfig;
use crate::common::context::ToolContext;
use crate::common::result::ToolResult;

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
    use serde_json::json;
    use async_trait::async_trait;
    use crate::common::config::ToolConfig;
    use crate::common::context::ToolContext;
    use crate::common::result::{ToolResult, ToolMetadata as ResultMetadata};

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

        assert_eq!(tool.kind(), "mock");
        assert_eq!(tool.metadata().name, "Mock Tool");

        let valid_input = json!({ "valid": true });
        let invalid_input = json!({ "valid": false });

        assert!(tool.validate_input(&valid_input, &ToolContext::default()).is_ok());
        assert!(tool.validate_input(&invalid_input, &ToolContext::default()).is_err());

        let context = ToolContext::default();
        let result = tool.execute(valid_input.clone(), context).await.unwrap();

        let output: Value = result.output;
        assert_eq!(output.get("success").unwrap(), &json!(true));
        assert_eq!(output.get("input").unwrap(), &valid_input);
    }
}
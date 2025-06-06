use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use serde_json::Value;

use crate::core::tool::Tool;
use crate::common::context::ToolContext;
use crate::common::result::ToolResult;

/// 工具注册表
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// 创建新的工具注册表
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 注册工具
    pub fn register<T: Tool + 'static>(&mut self, tool: T) -> Result<()> {
        let kind = tool.kind().to_string();
        if self.tools.contains_key(&kind) {
            return Err(anyhow!("Tool {} already registered", kind));
        }
        self.tools.insert(kind, Arc::new(tool));
        Ok(())
    }

    /// 获取工具
    pub fn get(&self, kind: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(kind).cloned()
    }

    /// 执行工具
    pub async fn execute(&self, kind: &str, input: Value) -> Result<ToolResult> {
        let tool = self.get(kind)
            .ok_or_else(|| anyhow!("Tool {} not found", kind))?;
        
        let context = ToolContext::default();
        tool.validate_input(&input, &context)?;
        tool.execute(input, context).await
    }

    /// 列出所有已注册的工具
    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// 获取工具元数据
    pub fn get_metadata(&self, kind: &str) -> Option<Value> {
        self.get(kind).map(|tool| {
            let metadata = tool.metadata();
            serde_json::to_value(metadata).unwrap_or_default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;
    use crate::common::config::ToolConfig;
    use crate::common::result::ToolMetadata as ResultMetadata;

    struct MockTool;

    #[async_trait]
    impl Tool for MockTool {
        fn kind(&self) -> &'static str {
            "mock"
        }

        fn metadata(&self) -> crate::core::tool::ToolMetadata {
            crate::core::tool::ToolMetadata {
                name: "Mock Tool".to_string(),
                description: "A mock tool for testing".to_string(),
                version: "1.0.0".to_string(),
                author: "Test".to_string(),
                tags: vec!["test".to_string()],
            }
        }

        fn default_config(&self) -> ToolConfig {
            ToolConfig::default()
        }

        fn validate_input(&self, input: &Value, _context: &ToolContext) -> Result<()> {
            if input.get("valid").and_then(Value::as_bool).unwrap_or(false) {
                Ok(())
            } else {
                Err(anyhow!("Invalid input"))
            }
        }

        async fn execute(&self, input: Value, context: ToolContext) -> Result<ToolResult> {
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

    #[test]
    fn test_registry_basic_operations() {
        let mut registry = ToolRegistry::new();
        
        // 测试注册工具
        assert!(registry.register(MockTool).is_ok());
        
        // 测试重复注册
        assert!(registry.register(MockTool).is_err());
        
        // 测试获取工具
        assert!(registry.get("mock").is_some());
        assert!(registry.get("nonexistent").is_none());
        
        // 测试列出工具
        let tools = registry.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0], "mock");
        
        // 测试获取元数据
        let metadata = registry.get_metadata("mock").unwrap();
        assert_eq!(metadata.get("name").unwrap(), "Mock Tool");
    }

    #[tokio::test]
    async fn test_registry_execution() {
        let mut registry = ToolRegistry::new();
        registry.register(MockTool).unwrap();

        // 测试有效输入
        let valid_input = json!({ "valid": true });
        let result = registry.execute("mock", valid_input.clone()).await.unwrap();
        let output = result.output;
        assert_eq!(output.get("success").unwrap(), &json!(true));
        assert_eq!(output.get("input").unwrap(), &valid_input);

        // 测试无效输入
        let invalid_input = json!({ "valid": false });
        assert!(registry.execute("mock", invalid_input).await.is_err());

        // 测试不存在的工具
        let result = registry.execute("nonexistent", json!({})).await;
        assert!(result.is_err());
    }
} 
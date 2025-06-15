// global_registry.rs
use once_cell::sync::Lazy;
use std::sync::Arc;
use crate::core::registry::ToolRegistry;
use crate::tools::{http::HttpTool, shell::ShellTool, file::FileTool};

pub static GLOBAL_TOOL_REGISTRY: Lazy<Arc<ToolRegistry>> = Lazy::new(|| {
    let mut registry = ToolRegistry::new();
    registry.register(HttpTool::new(None)).unwrap();
    registry.register(ShellTool::new(None)).unwrap();
    registry.register(FileTool::new(None)).unwrap();
    Arc::new(registry)
});
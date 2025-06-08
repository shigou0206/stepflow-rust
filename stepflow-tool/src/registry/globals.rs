use once_cell::sync::Lazy;
use std::sync::Mutex;
use crate::core::registry::ToolRegistry;

pub static GLOBAL_TOOL_REGISTRY: Lazy<Mutex<ToolRegistry>> = Lazy::new(|| {
    let mut registry = ToolRegistry::new();

    // 注册内建工具
    use crate::tools::{http::HttpTool, shell::ShellTool, file::FileTool};

    registry.register(HttpTool::new(None)).unwrap();
    registry.register(ShellTool::new(None)).unwrap();
    registry.register(FileTool::new(None)).unwrap();

    Mutex::new(registry)
});
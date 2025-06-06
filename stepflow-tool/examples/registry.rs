use serde_json::json;
use stepflow_tool::tools::http::HttpTool;
use stepflow_tool::tools::shell::ShellTool;
use stepflow_tool::tools::file::FileTool;
use stepflow_tool::core::registry::ToolRegistry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建工具注册表
    let mut registry = ToolRegistry::new();

    // 注册工具
    registry.register(HttpTool::new(None))?;
    registry.register(ShellTool::new(None))?;
    registry.register(FileTool::new(None))?;

    // 列出所有已注册的工具
    println!("Registered tools:");
    for tool in registry.list_tools() {
        let metadata = registry.get_metadata(&tool).unwrap();
        println!("- {} ({})", metadata.get("name").unwrap(), tool);
    }

    // 执行 HTTP 请求
    println!("\nExecuting HTTP request...");
    let result = registry.execute("http", json!({
        "url": "https://httpbin.org/get",
        "method": "GET",
        "headers": {
            "User-Agent": "StepFlow-Example"
        }
    })).await?;
    println!("Response: {}", result.output);

    // 执行 Shell 命令
    println!("\nExecuting Shell command...");
    let result = registry.execute("shell", json!({
        "command": "echo 'Hello from Shell!'"
    })).await?;
    println!("Output: {}", result.output);

    // 执行文件操作
    println!("\nExecuting File operation...");
    let result = registry.execute("file", json!({
        "path": "test.txt",
        "operation": {
            "type": "Write",
            "content": "Hello from File!"
        }
    })).await?;
    println!("Result: {}", result.output);

    // 读取文件内容
    let result = registry.execute("file", json!({
        "path": "test.txt",
        "operation": {
            "type": "Read"
        }
    })).await?;
    println!("File content: {}", result.output.get("content").unwrap());

    Ok(())
} 
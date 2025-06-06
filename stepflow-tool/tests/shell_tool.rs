use serde_json::json;
use stepflow_tool::{Tool, ToolContext};
use stepflow_tool::tools::shell::{ShellTool, ShellConfig};
use std::collections::HashMap;
use std::path::PathBuf;

#[tokio::test]
async fn test_shell_echo_command() {
    let tool = ShellTool::new(None);
    let context = ToolContext::default();

    let input = json!({
        "command": "echo 'Hello World'",
    });

    let result = tool.execute(input, context).await.unwrap();
    let output = result.output;

    assert_eq!(output.get("exit_code").unwrap().as_i64().unwrap(), 0);
    assert_eq!(
        output.get("stdout").unwrap().as_str().unwrap().trim(),
        "Hello World"
    );
}

#[tokio::test]
async fn test_shell_with_env_vars() {
    let mut config = ShellConfig::default();
    let mut env = HashMap::new();
    env.insert("TEST_VAR".to_string(), "test_value".to_string());
    config.env = Some(env);

    let tool = ShellTool::new(Some(config));
    let context = ToolContext::default();

    let input = json!({
        "command": "echo $TEST_VAR",
    });

    let result = tool.execute(input, context).await.unwrap();
    let output = result.output;

    assert_eq!(output.get("exit_code").unwrap().as_i64().unwrap(), 0);
    assert_eq!(
        output.get("stdout").unwrap().as_str().unwrap().trim(),
        "test_value"
    );
}

#[tokio::test]
async fn test_shell_working_directory() {
    let mut config = ShellConfig::default();
    config.working_dir = Some(PathBuf::from("/tmp"));

    let tool = ShellTool::new(Some(config));
    let context = ToolContext::default();

    let input = json!({
        "command": "pwd",
    });

    let result = tool.execute(input, context).await.unwrap();
    let output = result.output;

    assert_eq!(output.get("exit_code").unwrap().as_i64().unwrap(), 0);
    let pwd = output.get("stdout").unwrap().as_str().unwrap().trim();
    assert!(pwd.ends_with("/tmp"), "Expected path to end with /tmp, got {}", pwd);
}

#[tokio::test]
async fn test_shell_error_command() {
    let tool = ShellTool::new(None);
    let context = ToolContext::default();

    let input = json!({
        "command": "nonexistent_command",
    });

    let result = tool.execute(input, context).await.unwrap();
    let output = result.output;

    assert_ne!(output.get("exit_code").unwrap().as_i64().unwrap(), 0);
    assert!(!output.get("stderr").unwrap().as_str().unwrap().is_empty());
} 
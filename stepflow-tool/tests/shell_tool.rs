use serde_json::json;
use stepflow_tool::{Tool, ToolContext};
use stepflow_tool::tools::shell::{ShellTool, ShellConfig};
use std::collections::HashMap;
use std::path::PathBuf;

fn build_payload(command: &str, extra: Option<serde_json::Value>) -> serde_json::Value {
    let mut parameters = json!({ "command": command });

    if let Some(extra_fields) = extra {
        if let Some(obj) = extra_fields.as_object() {
            for (k, v) in obj {
                parameters.as_object_mut().unwrap().insert(k.clone(), v.clone());
            }
        }
    }

    json!({
        "resource": "shell",
        "input": null,
        "parameters": parameters
    })
}

#[tokio::test]
async fn test_shell_echo_command() {
    let tool = ShellTool::new(None);
    let context = ToolContext::default();

    let input = build_payload("echo 'Hello World'", None);

    let result = tool.execute(input, context).await.unwrap();
    let output = result.output;

    assert_eq!(output.get("exit_code").unwrap().as_i64().unwrap(), 0);
    assert_eq!(output.get("stdout").unwrap().as_str().unwrap().trim(), "Hello World");
}

#[tokio::test]
async fn test_shell_with_env_vars() {
    let mut config = ShellConfig::default();
    config.env = Some({
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());
        env
    });

    let tool = ShellTool::new(Some(config));
    let context = ToolContext::default();

    let input = build_payload("echo $TEST_VAR", None);

    let result = tool.execute(input, context).await.unwrap();
    let output = result.output;

    assert_eq!(output.get("exit_code").unwrap().as_i64().unwrap(), 0);
    assert_eq!(output.get("stdout").unwrap().as_str().unwrap().trim(), "test_value");
}

#[tokio::test]
async fn test_shell_working_directory() {
    let mut config = ShellConfig::default();
    config.working_dir = Some(PathBuf::from("/tmp"));

    let tool = ShellTool::new(Some(config));
    let context = ToolContext::default();

    let input = build_payload("pwd", None);

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

    let input = build_payload("nonexistent_command", None);

    let result = tool.execute(input, context).await.unwrap();
    let output = result.output;

    assert_ne!(output.get("exit_code").unwrap().as_i64().unwrap(), 0);
    assert!(!output.get("stderr").unwrap().as_str().unwrap().is_empty());
}
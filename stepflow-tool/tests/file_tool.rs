use serde_json::json;
use stepflow_tool::{Tool, ToolContext};
use stepflow_tool::tools::file::{FileTool, FileConfig};
use tempfile::tempdir;

fn build_payload(path: &str, operation: serde_json::Value) -> serde_json::Value {
    json!({
        "resource": "file",
        "input": null,
        "parameters": {
            "path": path,
            "operation": operation
        }
    })
}

#[tokio::test]
async fn test_file_write_and_read() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let file_path_str = file_path.to_str().unwrap();

    let mut config = FileConfig::default();
    config.create_dirs = true;
    config.overwrite = true;

    let tool = FileTool::new(Some(config));
    let context = ToolContext::default();

    let write_payload = build_payload(
        file_path_str,
        json!({ "type": "Write", "content": "Hello, World!" })
    );

    let result = tool.execute(write_payload, context.clone()).await.unwrap();
    assert!(result.output.get("success").unwrap().as_bool().unwrap());

    let read_payload = build_payload(
        file_path_str,
        json!({ "type": "Read" })
    );

    let result = tool.execute(read_payload, context).await.unwrap();
    let output = result.output;

    assert!(output.get("success").unwrap().as_bool().unwrap());
    assert_eq!(output.get("content").unwrap().as_str().unwrap(), "Hello, World!");
}

#[tokio::test]
async fn test_file_list_and_delete() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let file_path_str = file_path.to_str().unwrap();
    let dir_path_str = temp_dir.path().to_str().unwrap();

    let tool = FileTool::new(None);
    let context = ToolContext::default();

    let write_payload = build_payload(
        file_path_str,
        json!({ "type": "Write", "content": "Test content" })
    );

    tool.execute(write_payload, context.clone()).await.unwrap();

    let list_payload = build_payload(
        dir_path_str,
        json!({ "type": "List", "pattern": "test" })
    );

    let result = tool.execute(list_payload, context.clone()).await.unwrap();
    let files = result.output.get("files").unwrap().as_array().unwrap();
    assert_eq!(files.len(), 1);

    let delete_payload = build_payload(
        file_path_str,
        json!({ "type": "Delete" })
    );

    let result = tool.execute(delete_payload, context).await.unwrap();
    assert!(result.output.get("success").unwrap().as_bool().unwrap());
}

#[tokio::test]
async fn test_file_copy_and_move() {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("source.txt");
    let copy_path = temp_dir.path().join("copy.txt");
    let move_path = temp_dir.path().join("moved.txt");

    let source_str = source_path.to_str().unwrap();
    let copy_str = copy_path.to_str().unwrap();
    let move_str = move_path.to_str().unwrap();

    let tool = FileTool::new(None);
    let context = ToolContext::default();

    let write_payload = build_payload(
        source_str,
        json!({ "type": "Write", "content": "Test content" })
    );
    tool.execute(write_payload, context.clone()).await.unwrap();

    let copy_payload = build_payload(
        source_str,
        json!({ "type": "Copy", "target": copy_str })
    );
    let result = tool.execute(copy_payload, context.clone()).await.unwrap();
    assert!(result.output.get("success").unwrap().as_bool().unwrap());

    let move_payload = build_payload(
        copy_str,
        json!({ "type": "Move", "target": move_str })
    );
    let result = tool.execute(move_payload, context).await.unwrap();
    assert!(result.output.get("success").unwrap().as_bool().unwrap());
}
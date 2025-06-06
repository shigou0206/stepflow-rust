use serde_json::json;
use stepflow_tool::{Tool, ToolContext};
use stepflow_tool::tools::file::{FileTool, FileConfig};
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::test]
async fn test_file_write_and_read() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    let mut config = FileConfig::default();
    config.create_dirs = true;
    config.overwrite = true;

    let tool = FileTool::new(Some(config));
    let context = ToolContext::default();

    // 写入文件
    let write_input = json!({
        "path": file_path.to_str().unwrap(),
        "operation": {
            "type": "Write",
            "content": "Hello, World!"
        }
    });

    let result = tool.execute(write_input, context.clone()).await.unwrap();
    assert!(result.output.get("success").unwrap().as_bool().unwrap());

    // 读取文件
    let read_input = json!({
        "path": file_path.to_str().unwrap(),
        "operation": {
            "type": "Read"
        }
    });

    let result = tool.execute(read_input, context).await.unwrap();
    let output = result.output;

    assert!(output.get("success").unwrap().as_bool().unwrap());
    assert_eq!(
        output.get("content").unwrap().as_str().unwrap(),
        "Hello, World!"
    );
}

#[tokio::test]
async fn test_file_list_and_delete() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    let tool = FileTool::new(None);
    let context = ToolContext::default();

    // 创建文件
    let write_input = json!({
        "path": file_path.to_str().unwrap(),
        "operation": {
            "type": "Write",
            "content": "Test content"
        }
    });

    tool.execute(write_input, context.clone()).await.unwrap();

    // 列出目录
    let list_input = json!({
        "path": temp_dir.path().to_str().unwrap(),
        "operation": {
            "type": "List",
            "pattern": "test"
        }
    });

    let result = tool.execute(list_input, context.clone()).await.unwrap();
    let output = result.output;

    assert!(output.get("success").unwrap().as_bool().unwrap());
    assert_eq!(output.get("files").unwrap().as_array().unwrap().len(), 1);

    // 删除文件
    let delete_input = json!({
        "path": file_path.to_str().unwrap(),
        "operation": {
            "type": "Delete"
        }
    });

    let result = tool.execute(delete_input, context).await.unwrap();
    assert!(result.output.get("success").unwrap().as_bool().unwrap());
}

#[tokio::test]
async fn test_file_copy_and_move() {
    let temp_dir = tempdir().unwrap();
    let source_path = temp_dir.path().join("source.txt");
    let copy_path = temp_dir.path().join("copy.txt");
    let move_path = temp_dir.path().join("moved.txt");

    let tool = FileTool::new(None);
    let context = ToolContext::default();

    // 创建源文件
    let write_input = json!({
        "path": source_path.to_str().unwrap(),
        "operation": {
            "type": "Write",
            "content": "Test content"
        }
    });

    tool.execute(write_input, context.clone()).await.unwrap();

    // 复制文件
    let copy_input = json!({
        "path": source_path.to_str().unwrap(),
        "operation": {
            "type": "Copy",
            "target": copy_path.to_str().unwrap()
        }
    });

    let result = tool.execute(copy_input, context.clone()).await.unwrap();
    assert!(result.output.get("success").unwrap().as_bool().unwrap());

    // 移动文件
    let move_input = json!({
        "path": copy_path.to_str().unwrap(),
        "operation": {
            "type": "Move",
            "target": move_path.to_str().unwrap()
        }
    });

    let result = tool.execute(move_input, context).await.unwrap();
    assert!(result.output.get("success").unwrap().as_bool().unwrap());
} 
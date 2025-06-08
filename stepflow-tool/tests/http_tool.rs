use serde_json::json;
use stepflow_tool::{Tool, ToolContext};
use stepflow_tool::tools::http::{HttpTool, HttpConfig};

#[tokio::test]
async fn test_http_get_real_url() {
    let tool = HttpTool::new(None);
    let context = ToolContext::default();

    let input = json!({
        "resource": "http",
        "input": null,
        "parameters": {
            "url": "https://jsonplaceholder.typicode.com/posts/1",
            "method": "GET"
        }
    });

    let result = tool.execute(input, context).await.unwrap();
    let output = result.output;

    assert_eq!(output.get("status").unwrap().as_u64().unwrap(), 200);
    assert!(output.get("body").unwrap().get("id").is_some());
}

#[tokio::test]
async fn test_http_post_with_body() {
    let tool = HttpTool::new(None);
    let context = ToolContext::default();

    let input = json!({
        "resource": "http",
        "input": null,
        "parameters": {
            "url": "https://jsonplaceholder.typicode.com/posts",
            "method": "POST",
            "headers": {
                "Content-Type": "application/json"
            },
            "body": {
                "title": "foo",
                "body": "bar",
                "userId": 1
            }
        }
    });

    let result = tool.execute(input, context).await.unwrap();
    let output = result.output;

    assert_eq!(output.get("status").unwrap().as_u64().unwrap(), 201);
    assert_eq!(output.get("body").unwrap().get("title").unwrap(), "foo");
}

#[tokio::test]
async fn test_http_invalid_url() {
    let tool = HttpTool::new(None);
    let context = ToolContext::default();

    let input = json!({
        "resource": "http",
        "input": null,
        "parameters": {
            "url": "http://invalid_url.local",
            "method": "GET"
        }
    });

    let result = tool.execute(input, context).await;
    assert!(result.is_err()); // expect error on bad host
}

#[tokio::test]
async fn test_http_with_query_and_headers() {
    let tool = HttpTool::new(None);
    let context = ToolContext::default();

    let input = json!({
        "resource": "http",
        "input": null,
        "parameters": {
            "url": "https://jsonplaceholder.typicode.com/comments",
            "method": "GET",
            "query": {
                "postId": "1"
            },
            "headers": {
                "Accept": "application/json"
            }
        }
    });

    let result = tool.execute(input, context).await.unwrap();
    let output = result.output;

    assert_eq!(output.get("status").unwrap(), 200);
    assert!(output.get("body").unwrap().is_array());
}

#[tokio::test]
async fn test_http_missing_parameters_should_fail() {
    let tool = HttpTool::new(None);
    let context = ToolContext::default();

    // 缺少 parameters 字段
    let input = json!({
        "resource": "http",
        "input": null
    });

    let result = tool.execute(input, context).await;
    assert!(result.is_err());
}
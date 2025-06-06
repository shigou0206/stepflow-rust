use serde_json::json;
use stepflow_tool::{Tool, ToolContext};
use stepflow_tool::tools::http::{HttpTool, HttpConfig};
use std::collections::HashMap;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_http_get_request() {
    // 启动模拟服务器
    let mock_server = MockServer::start().await;

    // 设置模拟响应
    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "message": "success"
        })))
        .mount(&mock_server)
        .await;

    // 创建 HTTP 工具
    let tool = HttpTool::new(None);
    let context = ToolContext::default();

    // 构建请求
    let input = json!({
        "url": format!("{}/test", mock_server.uri()),
        "method": "GET",
        "headers": {
            "Content-Type": "application/json"
        }
    });

    // 执行请求
    let result = tool.execute(input, context).await.unwrap();
    let output = result.output;

    // 验证响应
    assert_eq!(output.get("status").unwrap().as_u64().unwrap(), 200);
    assert_eq!(
        output.get("body").unwrap().get("message").unwrap().as_str().unwrap(),
        "success"
    );
}

#[tokio::test]
async fn test_http_post_request() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": 1,
            "status": "created"
        })))
        .mount(&mock_server)
        .await;

    let mut config = HttpConfig::default();
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "StepFlow-Test".to_string());
    config.headers = Some(headers);

    let tool = HttpTool::new(Some(config));
    let context = ToolContext::default();

    let input = json!({
        "url": format!("{}/test", mock_server.uri()),
        "method": "POST",
        "headers": {
            "Content-Type": "application/json"
        },
        "body": {
            "name": "test",
            "value": 123
        }
    });

    let result = tool.execute(input, context).await.unwrap();
    let output = result.output;

    assert_eq!(output.get("status").unwrap().as_u64().unwrap(), 201);
    assert_eq!(
        output.get("body").unwrap().get("status").unwrap().as_str().unwrap(),
        "created"
    );
}

#[tokio::test]
async fn test_http_error_handling() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/error"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": "Not Found"
        })))
        .mount(&mock_server)
        .await;

    let tool = HttpTool::new(None);
    let context = ToolContext::default();

    let input = json!({
        "url": format!("{}/error", mock_server.uri()),
        "method": "GET"
    });

    let result = tool.execute(input, context).await.unwrap();
    let output = result.output;

    assert_eq!(output.get("status").unwrap().as_u64().unwrap(), 404);
    assert_eq!(
        output.get("body").unwrap().get("error").unwrap().as_str().unwrap(),
        "Not Found"
    );
} 
use serde_json::Value;

/// 执行 HTTP GET 请求
pub async fn get(input: &Value) -> Result<Value, String> {
    // 支持参数: { "url": "...", "headers": {...} }
    let url = input.get("url").and_then(Value::as_str).ok_or("missing url")?;

    let client = reqwest::Client::new();
    let mut req = client.get(url);

    // 可选支持 headers
    if let Some(headers) = input.get("headers").and_then(Value::as_object) {
        for (k, v) in headers {
            if let Some(s) = v.as_str() {
                req = req.header(k, s);
            }
        }
    }

    let resp = req.send().await.map_err(|e| format!("http.get error: {}", e))?;
    let json = resp.json::<Value>().await.map_err(|e| format!("invalid JSON: {}", e))?;

    Ok(json)
}

/// 执行 HTTP POST 请求
pub async fn post(input: &Value) -> Result<Value, String> {
    let url = input.get("url").and_then(Value::as_str).ok_or("missing url")?;
    let body = input.get("body").cloned().unwrap_or_else(|| Value::Object(Default::default()));

    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("http.post error: {}", e))?;

    let json = resp.json::<Value>().await.map_err(|e| format!("invalid JSON: {}", e))?;
    Ok(json)
} 
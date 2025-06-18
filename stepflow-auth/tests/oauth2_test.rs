use stepflow_auth::{AuthSpec, resolve_token};
use stepflow_mapping::model::{MappingDSL, MappingRule, MappingType};
use stepflow_mapping::engine::context::MappingContext;
use serde_json::{Map, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_oauth2_token() {
    // 🧪 启动本地 mock server
    let server = MockServer::start().await;

    // 🧪 注册 mock 响应
    let token_value = "mock_access_token_xyz";
    let mock_response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "access_token": token_value,
        "expires_in": 3600,
        "refresh_token": "mock_refresh"
    }));
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(mock_response)
        .mount(&server)
        .await;

    // ✅ 构造 DSL
    let dsl = MappingDSL {
        version: Some("1.0".into()),
        mappings: vec![
            MappingRule {
                key: "token_url".into(),
                mapping_type: MappingType::Constant,
                value: Some(Value::String(format!("{}/token", server.uri()))),
                ..Default::default()
            },
            MappingRule {
                key: "client_id".into(),
                mapping_type: MappingType::Constant,
                value: Some(Value::String("abc".into())),
                ..Default::default()
            },
            MappingRule {
                key: "client_secret".into(),
                mapping_type: MappingType::Constant,
                value: Some(Value::String("xyz".into())),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let auth = AuthSpec {
        r#type: "oauth2".into(),
        fields: dsl,
        inject: None,
    };

    let ctx = MappingContext::new(Map::new());

    let token = resolve_token(&auth, &ctx).await.unwrap();
    assert_eq!(token.access_token, token_value);
    assert!(token.expires_at.is_some());
    assert_eq!(token.refresh_token.as_deref(), Some("mock_refresh"));
}
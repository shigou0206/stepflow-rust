use stepflow_auth::{AuthSpec, resolve_token};
use stepflow_mapping::model::{MappingDSL, MappingRule, MappingType};
use stepflow_mapping::engine::context::MappingContext;
use serde_json::{Map, Value};

#[tokio::test]
async fn test_apikey_token() {
    let dsl = MappingDSL {
        version: Some("1.0".into()),
        mappings: vec![MappingRule {
            key: "value".into(),  // API Key 类型需要 key 为 "value"
            mapping_type: MappingType::Constant,
            value: Some(Value::String("my-api-key".into())),
            ..Default::default()
        }],
        ..Default::default()
    };

    let auth = AuthSpec {
        r#type: "apikey".into(),
        fields: dsl,
        inject: None,
    };

    let ctx = MappingContext::new(Map::new());
    let token = resolve_token(&auth, &ctx).await.unwrap();
    assert_eq!(token.access_token, "my-api-key");
}
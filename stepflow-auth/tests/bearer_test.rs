use stepflow_auth::{AuthSpec, resolve_token};
use stepflow_mapping::model::{MappingDSL, MappingRule, MappingType};
use stepflow_mapping::engine::context::MappingContext;
use serde_json::{Map, Value};
use tokio;

#[tokio::test]
async fn test_bearer_token() {
    let dsl = MappingDSL {
        version: Some("1.0".into()),
        mappings: vec![MappingRule {
            key: "token".into(),
            mapping_type: MappingType::Constant,
            value: Some(Value::String("abc123".into())),
            ..Default::default()
        }],
        ..Default::default()
    };

    let auth = AuthSpec {
        r#type: "bearer".into(),
        fields: dsl,
        inject: None,
    };

    let ctx = MappingContext::new(Map::new());

    let token = resolve_token(&auth, &ctx).await.unwrap();
    assert_eq!(token.access_token, "abc123");
}
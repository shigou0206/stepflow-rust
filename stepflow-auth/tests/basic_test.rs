use stepflow_auth::{AuthSpec, resolve_token};
use stepflow_mapping::model::{MappingDSL, MappingRule, MappingType};
use stepflow_mapping::engine::context::MappingContext;
use serde_json::{Map, Value};
use base64::{engine::general_purpose, Engine as _};

#[tokio::test]
async fn test_basic_token() {
    let username = "user";
    let password = "pass";
    let expected_encoded = general_purpose::STANDARD.encode(format!("{}:{}", username, password));

    let dsl = MappingDSL {
        version: Some("1.0".into()),
        mappings: vec![
            MappingRule {
                key: "username".into(),
                mapping_type: MappingType::Constant,
                value: Some(Value::String(username.into())),
                ..Default::default()
            },
            MappingRule {
                key: "password".into(),
                mapping_type: MappingType::Constant,
                value: Some(Value::String(password.into())),
                ..Default::default()
            }
        ],
        ..Default::default()
    };

    let auth = AuthSpec {
        r#type: "basic".into(),
        fields: dsl,
        inject: None,
    };

    let ctx = MappingContext::new(Map::new());

    let token = resolve_token(&auth, &ctx).await.unwrap();
    assert_eq!(token.access_token, expected_encoded);
}
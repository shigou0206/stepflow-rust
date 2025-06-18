use stepflow_auth::{inject_token, model::{InjectTarget, TokenResult}, injector::HttpRequestParts};
use std::collections::HashMap;
use chrono::{Utc, Duration};

#[test]
fn test_inject_token_header() {
    let mut request = HttpRequestParts {
        headers: HashMap::new(),
        query: HashMap::new(),
        body: HashMap::new(),
    };

    let token = TokenResult {
        access_token: "abc123".into(),
        expires_at: Some(Utc::now() + Duration::seconds(3600)),
        refresh_token: None,
    };

    let inject = InjectTarget::Header {
        header_name: "Authorization".into(),
        format: "Bearer ${access_token}".into(),
    };

    inject_token(&mut request, &token, &inject).unwrap();

    assert_eq!(
        request.headers.get("Authorization").unwrap(),
        "Bearer abc123"
    );
}

#[test]
fn test_inject_token_query() {
    let mut request = HttpRequestParts {
        headers: HashMap::new(),
        query: HashMap::new(),
        body: HashMap::new(),
    };

    let token = TokenResult {
        access_token: "xyz789".into(),
        expires_at: None,
        refresh_token: None,
    };

    let inject = InjectTarget::Query {
        key: "token".into(),
        format: "${access_token}".into(),
    };

    inject_token(&mut request, &token, &inject).unwrap();

    assert_eq!(request.query.get("token").unwrap(), "xyz789");
}

#[test]
fn test_inject_token_body() {
    let mut request = HttpRequestParts {
        headers: HashMap::new(),
        query: HashMap::new(),
        body: HashMap::new(),
    };

    let token = TokenResult {
        access_token: "abcd".into(),
        expires_at: None,
        refresh_token: Some("r1".into()),
    };

    let inject = InjectTarget::Body {
        key: "access".into(),
        format: "token=${access_token}".into(),
    };

    inject_token(&mut request, &token, &inject).unwrap();

    assert_eq!(request.body.get("access").unwrap(), "token=abcd");
}
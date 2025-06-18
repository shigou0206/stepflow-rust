use crate::model::TokenResult;
use crate::error::AuthError;
use serde_json::Value;
use std::collections::HashMap;
use base64::{engine::general_purpose, Engine as _};

pub fn get_token(fields: &HashMap<String, Value>) -> Result<TokenResult, AuthError> {
    let username = fields
        .get("username")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::MissingField("username".into()))?;
    let password = fields
        .get("password")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::MissingField("password".into()))?;

    let credentials = format!("{}:{}", username, password);
    let encoded = general_purpose::STANDARD.encode(credentials);

    Ok(TokenResult {
        access_token: encoded,
        expires_at: None,
        refresh_token: None,
    })
}

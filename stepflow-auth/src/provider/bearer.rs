use crate::model::TokenResult;
use crate::error::AuthError;
use serde_json::Value;
use std::collections::HashMap;

pub fn get_token(fields: &HashMap<String, Value>) -> Result<TokenResult, AuthError> {
    let token = fields
        .get("token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::MissingField("token".into()))?;

    Ok(TokenResult {
        access_token: token.to_string(),
        expires_at: None,
        refresh_token: None,
    })
}

use crate::model::TokenResult;
use crate::error::AuthError;
use serde_json::Value;
use std::collections::HashMap;
use reqwest::Client;

pub async fn get_token(fields: &HashMap<String, Value>) -> Result<TokenResult, AuthError> {
    let token_url = fields
        .get("token_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::MissingField("token_url".into()))?;
    let client_id = fields
        .get("client_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::MissingField("client_id".into()))?;
    let client_secret = fields
        .get("client_secret")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AuthError::MissingField("client_secret".into()))?;

    let grant_type = fields
        .get("grant_type")
        .and_then(|v| v.as_str())
        .unwrap_or("client_credentials");

    let form = [
        ("grant_type", grant_type),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];

    let response = Client::new()
        .post(token_url)
        .form(&form)
        .send()
        .await
        .map_err(|e| AuthError::HttpError(e.to_string()))?;

    let json: Value = response
        .json()
        .await
        .map_err(|_| AuthError::TokenParseError)?;

    let access_token = json
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or(AuthError::TokenParseError)?
        .to_string();

    let refresh_token = json.get("refresh_token").and_then(|v| v.as_str()).map(|s| s.to_string());
    let expires_at = json.get("expires_in")
        .and_then(|v| v.as_i64())
        .map(|sec| chrono::Utc::now() + chrono::Duration::seconds(sec));

    Ok(TokenResult {
        access_token,
        refresh_token,
        expires_at,
    })
}

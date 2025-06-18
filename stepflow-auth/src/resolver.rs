use crate::{model::AuthSpec, model::TokenResult, error::AuthError};
use crate::provider;
use stepflow_mapping::{engine::MappingEngine, engine::context::MappingContext};
use serde_json::Value;
use std::collections::HashMap;

pub async fn resolve_token(
    spec: &AuthSpec,
    ctx: &MappingContext,
) -> Result<TokenResult, AuthError> {
    let input = ctx.to_json();
    let value = MappingEngine::apply(spec.fields.clone(), &input)
        .map_err(|e| AuthError::MappingError(e.to_string()))?;

        let field_map: HashMap<String, Value> = value.as_object()
        .cloned()
        .map(|m| m.into_iter().collect())
        .ok_or_else(|| AuthError::MappingError("mapping output not object".into()))?;

    match spec.r#type.as_str() {
        "bearer" => provider::bearer::get_token(&field_map),
        "apikey" => provider::apikey::get_token(&field_map),
        "basic" => provider::basic::get_token(&field_map),
        "oauth2" => provider::oauth2::get_token(&field_map).await,
        other => Err(AuthError::UnsupportedType(other.to_string())),
    }
}

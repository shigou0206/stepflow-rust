use crate::model::{InjectTarget, TokenResult};
use crate::error::AuthError;
use std::collections::HashMap;

pub struct HttpRequestParts {
    pub headers: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub body: HashMap<String, String>,
}

pub fn inject_token(
    request: &mut HttpRequestParts,
    token: &TokenResult,
    inject: &InjectTarget,
) -> Result<(), AuthError> {
    let formatted = |tpl: &str| tpl.replace("${access_token}", &token.access_token);

    match inject {
        InjectTarget::Header { header_name, format } => {
            request.headers.insert(header_name.clone(), formatted(format));
        }
        InjectTarget::Query { key, format } => {
            request.query.insert(key.clone(), formatted(format));
        }
        InjectTarget::Body { key, format } => {
            request.body.insert(key.clone(), formatted(format));
        }
    }

    Ok(())
}

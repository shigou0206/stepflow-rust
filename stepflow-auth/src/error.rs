use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Unsupported auth type: {0}")]
    UnsupportedType(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Mapping execution failed: {0}")]
    MappingError(String),

    #[error("Invalid token response format")]
    TokenParseError,

    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("Token injection failed: {0}")]
    InjectError(String),

    #[error("Internal auth error")]
    Internal,
}

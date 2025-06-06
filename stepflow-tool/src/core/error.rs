use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool execution timeout")]
    Timeout,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Tool not found: {0}")]
    NotFound(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type ToolResult<T> = Result<T, ToolError>; 
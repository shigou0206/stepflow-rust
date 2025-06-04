use thiserror::Error;

#[derive(Error, Debug)]
pub enum PostgresError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Entity not found")]
    NotFound,
} 
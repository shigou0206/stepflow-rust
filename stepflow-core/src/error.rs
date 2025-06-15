use axum::{response::IntoResponse, Json};
use http::StatusCode;
use serde_json::json;

pub type AppResult<T> = Result<T, AppError>;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::NotFound      => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            _                       => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({"error":{"message":self.to_string()}}))).into_response()
    }
}
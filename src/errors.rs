use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("plant not found: {0}")]
    NotFound(String),
    #[error("internal store lock error")]
    LockError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(id) => (
                StatusCode::NOT_FOUND,
                format!("plant not found: {id}"),
            ),
            AppError::LockError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            ),
        };
        let body = Json(json!({ "error": message }));
        (status, body).into_response()
    }
}

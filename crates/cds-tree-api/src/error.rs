use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Application-level errors with HTTP responses
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Tree not found: {0}")]
    TreeNotFound(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Tree validation failed: {0}")]
    ValidationFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization denied")]
    Forbidden,

    #[error("Tree already exists: {0}")]
    Conflict(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal server error")]
    InternalError,
}

/// JSON error response body
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
    pub request_id: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match self {
            AppError::TreeNotFound(id) => (
                StatusCode::NOT_FOUND,
                "Tree not found".to_string(),
                Some(id),
            ),
            AppError::NodeNotFound(id) => (
                StatusCode::NOT_FOUND,
                "Node not found".to_string(),
                Some(id),
            ),
            AppError::SessionNotFound(id) => (
                StatusCode::NOT_FOUND,
                "Session not found".to_string(),
                Some(id),
            ),
            AppError::InvalidInput(msg) => (
                StatusCode::BAD_REQUEST,
                "Invalid input".to_string(),
                Some(msg),
            ),
            AppError::ValidationFailed(msg) => (
                StatusCode::BAD_REQUEST,
                "Validation failed".to_string(),
                Some(msg),
            ),
            AppError::AuthenticationFailed(msg) => (
                StatusCode::UNAUTHORIZED,
                "Authentication failed".to_string(),
                Some(msg),
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "Access denied".to_string(),
                None,
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                "Conflict".to_string(),
                Some(msg),
            ),
            AppError::DatabaseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
                Some(msg),
            ),
            AppError::InternalError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
                None,
            ),
        };

        let body = Json(json!({
            "error": error_message,
            "details": details
        }));

        (status, body).into_response()
    }
}

/// Result type alias for API handlers
pub type ApiResult<T> = Result<T, AppError>;

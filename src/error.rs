use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] turso::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl ApiError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound(resource.into())
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            ApiError::Database(ref err) => {
                // Log the actual database error for debugging but don't expose it
                tracing::error!("Database error: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "An internal database error occurred".to_string(),
                )
            }
            ApiError::Validation(ref message) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                message.clone(),
            ),
            ApiError::NotFound(ref resource) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                format!("{} not found", resource),
            ),
            ApiError::Conflict(ref message) => (
                StatusCode::CONFLICT,
                "CONFLICT",
                message.clone(),
            ),
            ApiError::Internal(ref err) => {
                // Log the actual internal error for debugging but don't expose it
                tracing::error!("Internal error: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "An internal server error occurred".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}

// Result type alias for convenience
pub type ApiResult<T> = Result<T, ApiError>;
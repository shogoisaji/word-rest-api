// Handlers module
// HTTP handlers for the REST API

pub mod users;
pub mod posts;

use axum::{http::StatusCode, response::IntoResponse};

/// Health check handler
/// Returns "OK" with 200 status for monitoring purposes
/// Requirements: 1.1, 1.2, 1.3
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
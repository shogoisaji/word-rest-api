// Handlers module
// HTTP handlers for the REST API

pub mod users;
pub mod posts;
pub mod vocabulary;

use axum::{http::StatusCode, response::IntoResponse};

/// Health check handler
/// Returns a friendly greeting with 200 status for monitoring purposes
/// Requirements: 1.1, 1.2, 1.3
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Hello Rust, Axum and Neon! 🚀")
}
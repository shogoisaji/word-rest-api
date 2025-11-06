// Handlers module
// HTTP handlers for the REST API

pub mod users;
pub mod posts;
pub mod vocabulary;

use axum::{http::StatusCode, response::IntoResponse};

/// ヘルスチェック用ハンドラ。
/// 200 OK と短いメッセージを返すだけだが、監視ツールや Cloud Run の
/// ヘルスプローブにそのまま利用できる。
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Hello Rust, Axum and Neon! 🚀")
}

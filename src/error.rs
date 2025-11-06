use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tokio_postgres::error::SqlState;

/// REST API 全体で共通利用するエラー型。
/// `thiserror::Error` を derive することで `?` 演算子と相性の良い独自エラーを簡潔に書ける。
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(String),
    
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
    /// バリデーションエラーを簡潔に生成するヘルパー。
    /// 型パラメータ `impl Into<String>` により、`&str`/`String` どちらも渡せる。
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    /// `NotFound` バリアントを作るユーティリティ。
    /// `resource` には「User 123」のような文言を入れておくとレスポンスにも反映される。
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound(resource.into())
    }

    /// 楽観ロックや一意制約違反を表すエラーを生成する。
    /// メッセージはそのままクライアントに返る点に注意。
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }
}

impl IntoResponse for ApiError {
    /// Axum の `IntoResponse` を実装することで、`Result<_, ApiError>` をそのままハンドラの戻り値にできる。
    /// ここでは HTTP ステータス・エラーコード・ユーザー向けメッセージを一括で決定している。
    fn into_response(self) -> Response {
        let (status, error_code, message) = match self {
            ApiError::Database(ref err) => {
                // Enhanced logging for PostgreSQL context without exposing sensitive details
                if err.contains("connection") {
                    tracing::error!("PostgreSQL connection issue: {}", err);
                } else if err.contains("timeout") {
                    tracing::warn!("PostgreSQL operation timeout: {}", err);
                } else {
                    tracing::error!("PostgreSQL database error: {}", err);
                }
                
                // Provide user-friendly message without exposing internal details
                let user_message = if err.contains("timeout") {
                    "Database operation timed out, please try again"
                } else if err.contains("unavailable") || err.contains("connection") {
                    "Database service is temporarily unavailable"
                } else {
                    "A database error occurred"
                };
                
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    user_message.to_string(),
                )
            }
            ApiError::Validation(ref message) => {
                // Log validation errors at debug level for PostgreSQL context
                tracing::debug!("PostgreSQL validation error: {}", message);
                (
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_ERROR",
                    message.clone(),
                )
            }
            ApiError::NotFound(ref resource) => {
                tracing::debug!("Resource not found: {}", resource);
                (
                    StatusCode::NOT_FOUND,
                    "NOT_FOUND",
                    format!("{} not found", resource),
                )
            }
            ApiError::Conflict(ref message) => {
                // Log conflict errors for PostgreSQL constraint violations
                tracing::debug!("PostgreSQL constraint conflict: {}", message);
                (
                    StatusCode::CONFLICT,
                    "CONFLICT",
                    message.clone(),
                )
            }
            ApiError::Internal(ref err) => {
                // Enhanced internal error logging with context
                tracing::error!("Internal server error in PostgreSQL context: {}", err);
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

// PostgreSQL error mapping
/// `tokio_postgres::Error` を `ApiError` に読み替える実装。
/// SQLSTATE に応じて適切なバリアントへマッピングすることで、重複や外部キー違反を分かりやすく返す。
impl From<tokio_postgres::Error> for ApiError {
    fn from(err: tokio_postgres::Error) -> Self {
        match err.code() {
            Some(&SqlState::UNIQUE_VIOLATION) => {
                // Check if it's an email constraint violation by examining the error message
                let message = if err.to_string().contains("email") {
                    "Email address already exists".to_string()
                } else {
                    "Resource already exists".to_string()
                };
                ApiError::Conflict(message)
            }
            Some(&SqlState::FOREIGN_KEY_VIOLATION) => {
                ApiError::Validation("Referenced resource does not exist".to_string())
            }
            Some(&SqlState::NOT_NULL_VIOLATION) => {
                // Extract column name from error message if possible
                let message = if err.to_string().contains("name") {
                    "Required field 'name' is missing".to_string()
                } else if err.to_string().contains("email") {
                    "Required field 'email' is missing".to_string()
                } else {
                    "Required field is missing".to_string()
                };
                ApiError::Validation(message)
            }
            Some(&SqlState::CHECK_VIOLATION) => {
                ApiError::Validation("Data validation constraint violated".to_string())
            }
            Some(&SqlState::INVALID_TEXT_REPRESENTATION) => {
                ApiError::Validation("Invalid data format provided".to_string())
            }
            Some(&SqlState::NUMERIC_VALUE_OUT_OF_RANGE) => {
                ApiError::Validation("Numeric value is out of range".to_string())
            }
            Some(&SqlState::STRING_DATA_LENGTH_MISMATCH) => {
                ApiError::Validation("Text data exceeds maximum length".to_string())
            }
            Some(&SqlState::CONNECTION_EXCEPTION) | 
            Some(&SqlState::CONNECTION_DOES_NOT_EXIST) |
            Some(&SqlState::CONNECTION_FAILURE) => {
                tracing::error!("PostgreSQL connection error: {}", err);
                ApiError::Database("Database connection unavailable".to_string())
            }
            Some(&SqlState::INSUFFICIENT_PRIVILEGE) => {
                tracing::error!("PostgreSQL privilege error: {}", err);
                ApiError::Database("Database access denied".to_string())
            }
            _ => {
                tracing::error!("Unhandled PostgreSQL error: {} (code: {:?})", err, err.code());
                ApiError::Database("Database operation failed".to_string())
            }
        }
    }
}

// Connection pool error mapping
/// Deadpool のプールエラーを REST 用のエラーに変換する。
/// タイムアウトやプール閉塞など、インフラ寄りの問題を `Database` エラーとして扱う。
impl From<deadpool_postgres::PoolError> for ApiError {
    fn from(err: deadpool_postgres::PoolError) -> Self {
        match err {
            deadpool_postgres::PoolError::Timeout(_) => {
                tracing::warn!("Database connection pool timeout: {}", err);
                ApiError::Database("Database connection timeout".to_string())
            }
            deadpool_postgres::PoolError::Closed => {
                tracing::error!("Database connection pool is closed: {}", err);
                ApiError::Database("Database service unavailable".to_string())
            }
            deadpool_postgres::PoolError::NoRuntimeSpecified => {
                tracing::error!("Database pool runtime error: {}", err);
                ApiError::Internal(anyhow::anyhow!("Database configuration error"))
            }
            deadpool_postgres::PoolError::PostCreateHook(_) => {
                tracing::error!("Database connection setup error: {}", err);
                ApiError::Database("Database connection setup failed".to_string())
            }
            _ => {
                tracing::error!("Database connection pool error: {}", err);
                ApiError::Database("Database connection unavailable".to_string())
            }
        }
    }
}

// Result type alias for convenience
pub type ApiResult<T> = Result<T, ApiError>;

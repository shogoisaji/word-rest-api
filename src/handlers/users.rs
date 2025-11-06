// User handlers
// HTTP handlers for user management operations

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::{
    db::Database,
    error::ApiError,
    models::user::{CreateUserRequest, UpdateUserRequest},
};

/// `POST /api/users`
/// Axum の `State<Arc<Database>>`/`Json<T>` エクストラクタを使った典型的な作成ハンドラ。
/// `db.create_user` が `Result` を返すため、`?` で早期リターンできる。
pub async fn create_user(
    State(db): State<Arc<Database>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Creating new user with email: {}", request.email);
    
    let user = db.create_user(request).await?;
    
    info!("Successfully created user with id: {}", user.id);
    Ok((StatusCode::CREATED, Json(user)))
}

/// `GET /api/users/:id`
/// `Path<Uuid>` によって UUID の妥当性チェックを Axum に任せられる例。
pub async fn get_user_by_id(
    State(db): State<Arc<Database>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching user with id: {}", user_id);
    
    let user = db.get_user_by_id(&user_id.to_string()).await?;
    
    Ok((StatusCode::OK, Json(user)))
}

/// `GET /api/users`
/// 返り値は `Vec<User>` を JSON 化したもの。`info!` で件数をログに残している。
pub async fn get_all_users(
    State(db): State<Arc<Database>>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching all users");
    
    let users = db.get_all_users().await?;
    
    info!("Retrieved {} users", users.len());
    Ok((StatusCode::OK, Json(users)))
}

/// `PUT /api/users/:id`
/// `Json<UpdateUserRequest>` が Option フィールドを含む点に注目。
pub async fn update_user(
    State(db): State<Arc<Database>>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Updating user with id: {}", user_id);
    
    let user = db.update_user(&user_id.to_string(), request).await?;
    
    info!("Successfully updated user with id: {}", user_id);
    Ok((StatusCode::OK, Json(user)))
}

/// `DELETE /api/users/:id`
/// 削除成功時は `StatusCode::NO_CONTENT` を返し、HTTP 的な慣習に従ってボディなしで応答する。
pub async fn delete_user(
    State(db): State<Arc<Database>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Deleting user with id: {}", user_id);
    
    db.delete_user(&user_id.to_string()).await?;
    
    info!("Successfully deleted user with id: {} (cascade deleted associated posts)", user_id);
    Ok(StatusCode::NO_CONTENT)
}

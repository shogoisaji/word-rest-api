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

use crate::{
    db::Database,
    error::ApiError,
    models::user::{CreateUserRequest, UpdateUserRequest},
};

/// Create a new user
/// POST /api/users
/// Requirements: 2.1, 2.2, 2.3, 2.4, 2.5
pub async fn create_user(
    State(db): State<Arc<Database>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Creating new user with email: {}", request.email);
    
    let user = db.create_user(request).await?;
    
    info!("Successfully created user with id: {}", user.id);
    Ok((StatusCode::CREATED, Json(user)))
}

/// Get user by ID
/// GET /api/users/:id
/// Requirements: 3.1, 3.2, 3.5
pub async fn get_user_by_id(
    State(db): State<Arc<Database>>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching user with id: {}", user_id);
    
    let user = db.get_user_by_id(&user_id).await?;
    
    Ok((StatusCode::OK, Json(user)))
}

/// Get all users
/// GET /api/users
/// Requirements: 3.1, 3.2, 3.5
pub async fn get_all_users(
    State(db): State<Arc<Database>>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching all users");
    
    let users = db.get_all_users().await?;
    
    info!("Retrieved {} users", users.len());
    Ok((StatusCode::OK, Json(users)))
}

/// Update user by ID
/// PUT /api/users/:id
/// Requirements: 4.1, 4.2, 4.3, 4.4, 4.5
pub async fn update_user(
    State(db): State<Arc<Database>>,
    Path(user_id): Path<String>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Updating user with id: {}", user_id);
    
    let user = db.update_user(&user_id, request).await?;
    
    info!("Successfully updated user with id: {}", user_id);
    Ok((StatusCode::OK, Json(user)))
}

/// Delete user by ID
/// DELETE /api/users/:id
/// Requirements: 5.1, 5.2, 5.3, 5.4, 5.5
pub async fn delete_user(
    State(db): State<Arc<Database>>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Deleting user with id: {}", user_id);
    
    db.delete_user(&user_id).await?;
    
    info!("Successfully deleted user with id: {} (cascade deleted associated posts)", user_id);
    Ok(StatusCode::NO_CONTENT)
}
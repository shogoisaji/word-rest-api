// Post handlers
// HTTP handlers for post management operations

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

use crate::{
    db::Database,
    error::ApiError,
    models::post::CreatePostRequest,
};

/// Query parameters for listing posts
#[derive(Debug, Deserialize)]
pub struct ListPostsQuery {
    pub user_id: Option<String>,
}

/// Create a new post
/// POST /api/posts
/// Requirements: 6.1, 6.2, 6.4, 6.5
pub async fn create_post(
    State(db): State<Arc<Database>>,
    Json(request): Json<CreatePostRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Creating new post for user_id: {} with title: {}", request.user_id, request.title);
    
    let post = db.create_post(request).await?;
    
    info!("Successfully created post with id: {}", post.id);
    Ok((StatusCode::CREATED, Json(post)))
}

/// Get post by ID
/// GET /api/posts/:id
/// Requirements: 7.1, 7.2, 7.5
pub async fn get_post_by_id(
    State(db): State<Arc<Database>>,
    Path(post_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching post with id: {}", post_id);
    
    let post = db.get_post_by_id(&post_id).await?;
    
    Ok((StatusCode::OK, Json(post)))
}

/// Get all posts, optionally filtered by user_id
/// GET /api/posts?user_id=<id>
/// Requirements: 7.1, 7.3, 7.4, 7.5
pub async fn get_all_posts(
    State(db): State<Arc<Database>>,
    Query(params): Query<ListPostsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(ref user_id) = params.user_id {
        info!("Fetching posts for user_id: {}", user_id);
    } else {
        info!("Fetching all posts");
    }
    
    let posts = db.get_all_posts(params.user_id.as_deref()).await?;
    
    if let Some(user_id) = params.user_id {
        info!("Retrieved {} posts for user_id: {}", posts.len(), user_id);
    } else {
        info!("Retrieved {} posts", posts.len());
    }
    
    Ok((StatusCode::OK, Json(posts)))
}
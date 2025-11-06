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
use uuid::Uuid;

use crate::{
    db::Database,
    error::ApiError,
    models::post::CreatePostRequest,
};

/// `GET /api/posts` のクエリパラメータを表す構造体。
/// `Option<Uuid>` にすることで、存在しない場合は全件取得と同じ挙動になる。
#[derive(Debug, Deserialize)]
pub struct ListPostsQuery {
    pub user_id: Option<Uuid>,
}

/// `POST /api/posts`
/// リクエストボディは JSON として受け取り、`CreatePostRequest` のバリデーション結果に従う。
pub async fn create_post(
    State(db): State<Arc<Database>>,
    Json(request): Json<CreatePostRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Creating new post for user_id: {} with title: {}", request.user_id, request.title);
    
    let post = db.create_post(request).await?;
    
    info!("Successfully created post with id: {}", post.id);
    Ok((StatusCode::CREATED, Json(post)))
}

/// `GET /api/posts/:id`
/// パスパラメータを `Uuid` として受け取り、そのまま DB レイヤーへ委譲する。
pub async fn get_post_by_id(
    State(db): State<Arc<Database>>,
    Path(post_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching post with id: {}", post_id);
    
    let post = db.get_post_by_id(&post_id.to_string()).await?;
    
    Ok((StatusCode::OK, Json(post)))
}

/// `GET /api/posts?user_id=<id>`
/// クエリの有無でログメッセージを変える例。戻り値は常に 200 OK + JSON 配列。
pub async fn get_all_posts(
    State(db): State<Arc<Database>>,
    Query(params): Query<ListPostsQuery>,
) -> Result<impl IntoResponse, ApiError> {
    if let Some(ref user_id) = params.user_id {
        info!("Fetching posts for user_id: {}", user_id);
    } else {
        info!("Fetching all posts");
    }
    
    let posts = db.get_all_posts(params.user_id.as_ref().map(|id| id.to_string()).as_deref()).await?;
    
    if let Some(user_id) = params.user_id {
        info!("Retrieved {} posts for user_id: {}", posts.len(), user_id);
    } else {
        info!("Retrieved {} posts", posts.len());
    }
    
    Ok((StatusCode::OK, Json(posts)))
}

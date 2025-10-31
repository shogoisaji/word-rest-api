// Vocabulary handlers
// HTTP handlers for vocabulary management operations

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
    models::vocabulary::CreateVocabularyRequest,
};

/// Create a new vocabulary entry
/// POST /api/vocabulary
/// Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 3.3
pub async fn create_vocabulary(
    State(db): State<Arc<Database>>,
    Json(request): Json<CreateVocabularyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Creating new vocabulary entry: {} -> {}", request.en_word, request.ja_word);
    
    let vocabulary = db.create_vocabulary(request).await?;
    
    info!("Successfully created vocabulary entry with id: {}", vocabulary.id);
    Ok((StatusCode::CREATED, Json(vocabulary)))
}

/// Get vocabulary entry by ID
/// GET /api/vocabulary/:id
/// Requirements: 2.2, 2.3, 2.4, 3.3
pub async fn get_vocabulary_by_id(
    State(db): State<Arc<Database>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching vocabulary entry with id: {}", id);
    
    let vocabulary = db.get_vocabulary_by_id(id).await?;
    
    Ok((StatusCode::OK, Json(vocabulary)))
}

/// Get all vocabulary entries
/// GET /api/vocabulary
/// Requirements: 2.1, 2.3, 3.3
pub async fn get_all_vocabulary(
    State(db): State<Arc<Database>>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching all vocabulary entries");
    
    let vocabulary_list = db.get_all_vocabulary().await?;
    
    info!("Retrieved {} vocabulary entries", vocabulary_list.len());
    Ok((StatusCode::OK, Json(vocabulary_list)))
}

/// Get a random vocabulary entry
/// GET /api/vocabulary/random
/// Returns a single random vocabulary entry for practice
pub async fn get_random_vocabulary(
    State(db): State<Arc<Database>>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching random vocabulary entry");
    
    let vocabulary = db.get_random_vocabulary().await?;
    
    info!("Retrieved random vocabulary: {} -> {}", vocabulary.en_word, vocabulary.ja_word);
    Ok((StatusCode::OK, Json(vocabulary)))
}

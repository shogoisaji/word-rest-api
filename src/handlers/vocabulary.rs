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

/// `POST /api/vocabulary`
/// 英単語・和訳・例文を受け取って DB に保存する。`CreateVocabularyRequest` 内で入力検証を行う。
pub async fn create_vocabulary(
    State(db): State<Arc<Database>>,
    Json(request): Json<CreateVocabularyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Creating new vocabulary entry: {} -> {}", request.en_word, request.ja_word);
    
    let vocabulary = db.create_vocabulary(request).await?;
    
    info!("Successfully created vocabulary entry with id: {}", vocabulary.id);
    Ok((StatusCode::CREATED, Json(vocabulary)))
}

/// `GET /api/vocabulary/:id`
/// `Path<i32>` により、整数変換エラー時は Axum が自動で 400 を返す。
pub async fn get_vocabulary_by_id(
    State(db): State<Arc<Database>>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching vocabulary entry with id: {}", id);
    
    let vocabulary = db.get_vocabulary_by_id(id).await?;
    
    Ok((StatusCode::OK, Json(vocabulary)))
}

/// `GET /api/vocabulary`
/// 全件を配列で返す。`info!` で件数をログに残しておくと、モニタリング時に便利。
pub async fn get_all_vocabulary(
    State(db): State<Arc<Database>>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching all vocabulary entries");
    
    let vocabulary_list = db.get_all_vocabulary().await?;
    
    info!("Retrieved {} vocabulary entries", vocabulary_list.len());
    Ok((StatusCode::OK, Json(vocabulary_list)))
}

/// `GET /api/vocabulary/random`
/// 単語帳からランダムに 1 件取る。練習問題用のエンドポイント。
pub async fn get_random_vocabulary(
    State(db): State<Arc<Database>>,
) -> Result<impl IntoResponse, ApiError> {
    info!("Fetching random vocabulary entry");
    
    let vocabulary = db.get_random_vocabulary().await?;
    
    info!("Retrieved random vocabulary: {} -> {}", vocabulary.en_word, vocabulary.ja_word);
    Ok((StatusCode::OK, Json(vocabulary)))
}

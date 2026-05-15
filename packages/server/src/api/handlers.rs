use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::api::router::AppState;
use crate::assessment::assessor;
use crate::db::models::*;
use crate::db::repository;
use crate::embedding::similarity;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Unified API error type that can be converted into an HTTP response.
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => {
                AppError::NotFound("Resource not found".to_string())
            }
            other => AppError::Internal(other.to_string()),
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for AppError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        AppError::Internal(err.to_string())
    }
}

// ---------------------------------------------------------------------------
// Query parameter types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    page: Option<u32>,
    per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ListSkillsParams {
    page: Option<u32>,
    per_page: Option<u32>,
    category: Option<String>,
    sort: Option<String>,
    order: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    q: String,
    category: Option<String>,
    license: Option<String>,
    page: Option<u32>,
    per_page: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct SimilarityRequest {
    descriptions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SimilarityResponse {
    pub similarities: Vec<Vec<f64>>,
}

#[derive(Debug, Deserialize)]
pub struct AssessRequest {
    skill_content: String,
}

// ---------------------------------------------------------------------------
// Global: default page = 20, max page = 100
// ---------------------------------------------------------------------------

fn validate_pagination(page: Option<u32>, per_page: Option<u32>) -> (u32, u32) {
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    (page, per_page)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /api/v1/health
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status": "ok"})))
}

/// GET /api/v1/stats
pub async fn stats(State(state): State<AppState>) -> Result<Json<Stats>, AppError> {
    let stats = repository::get_stats(&state.db).await?;
    Ok(Json(stats))
}

/// GET /api/v1/skills
pub async fn list_skills(
    State(state): State<AppState>,
    Query(params): Query<ListSkillsParams>,
) -> Result<Json<PaginatedResponse<Skill>>, AppError> {
    let (page, per_page) = validate_pagination(params.page, params.per_page);

    let result = repository::list_skills(
        &state.db,
        page,
        per_page,
        params.category,
        params.sort,
        params.order,
    )
    .await?;

    Ok(Json(result))
}

/// GET /api/v1/skills/search?q=...
pub async fn search_skills(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<PaginatedResponse<Skill>>, AppError> {
    let (page, per_page) = validate_pagination(params.page, params.per_page);

    let result = repository::search_skills(
        &state.db,
        &params.q,
        params.category,
        params.license,
        page,
        per_page,
    )
    .await?;

    Ok(Json(result))
}

/// GET /api/v1/skills/:id
pub async fn get_skill_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SkillWithCategories>, AppError> {
    let skill = repository::get_skill_by_id(&state.db, id).await?;
    let categories = repository::get_categories_for_skill(&state.db, id).await?;

    Ok(Json(SkillWithCategories { skill, categories }))
}

/// GET /api/v1/skills/:id/similar
pub async fn get_similar_skills(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Skill>>, AppError> {
    let skills = repository::get_similar_skills(&state.db, id, 10).await?;
    Ok(Json(skills))
}

/// GET /api/v1/categories
pub async fn get_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<Category>>, AppError> {
    let categories = repository::get_categories(&state.db).await?;
    Ok(Json(categories))
}

/// POST /api/v1/skills/similarity
///
/// Accepts a list of descriptions and returns a TF-IDF cosine similarity matrix.
pub async fn similarity(
    State(_state): State<AppState>,
    Json(body): Json<SimilarityRequest>,
) -> Result<Json<SimilarityResponse>, AppError> {
    if body.descriptions.is_empty() {
        return Err(AppError::BadRequest(
            "descriptions must not be empty".to_string(),
        ));
    }

    let similarities = similarity::compute_similarities(body.descriptions);
    Ok(Json(SimilarityResponse { similarities }))
}

/// POST /api/v1/crawl/trigger
///
/// Sends a signal to the crawl scheduler to start a crawl cycle early.
pub async fn trigger_crawl(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    match state.crawl_tx.send(()) {
        Ok(_) => Ok(Json(json!({"message": "Crawl triggered successfully"}))),
        Err(_) => Ok(Json(
            json!({"message": "Crawl signal sent (no active listeners)"}),
        )),
    }
}

/// POST /api/v1/assess
///
/// Sends skill content to the configured LLM provider for quality assessment.
pub async fn assess(
    State(state): State<AppState>,
    Json(body): Json<AssessRequest>,
) -> Result<Json<assessor::AssessResult>, AppError> {
    if body.skill_content.trim().is_empty() {
        return Err(AppError::BadRequest(
            "skill_content must not be empty".to_string(),
        ));
    }

    let result = assessor::assess(&state.llm_provider, &body.skill_content).await?;
    Ok(Json(result))
}

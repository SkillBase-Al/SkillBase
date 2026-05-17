use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::StatusCode;
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::Json;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
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
    Unauthorized(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
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

#[derive(Debug, Deserialize)]
pub struct DateRangeParams {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PageViewRequest {
    pub page: String,
}

#[derive(Debug, Deserialize)]
pub struct FeedbackRequest {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
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

fn parse_date_range(params: &DateRangeParams) -> Result<(NaiveDate, NaiveDate), AppError> {
    let today = chrono::Utc::now().date_naive();
    let from = match &params.from {
        Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid from date, use YYYY-MM-DD".into()))?,
        None => today - chrono::Duration::days(30),
    };
    let to = match &params.to {
        Some(s) => NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("Invalid to date, use YYYY-MM-DD".into()))?,
        None => today,
    };
    if from > to {
        return Err(AppError::BadRequest("from must be before or equal to to".into()));
    }
    Ok((from, to))
}

// ---------------------------------------------------------------------------
// Telemetry Handlers
// ---------------------------------------------------------------------------

/// POST /api/v1/telemetry/heartbeat
pub async fn heartbeat(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ip = addr.ip().to_string();
    let today = chrono::Utc::now().date_naive();
    repository::upsert_dau(&state.db, &ip, today).await?;
    Ok(Json(json!({"status": "ok"})))
}

/// POST /api/v1/telemetry/pageview
pub async fn pageview(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<PageViewRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ip = addr.ip().to_string();
    repository::insert_pageview(&state.db, &ip, &body.page).await?;
    Ok(Json(json!({"status": "ok"})))
}

/// POST /api/v1/feedback
pub async fn submit_feedback(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<FeedbackRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ip = addr.ip().to_string();
    repository::insert_feedback(&state.db, &body.title, &body.description, Some(&ip)).await?;
    Ok(Json(json!({"status": "ok"})))
}

// ---------------------------------------------------------------------------
// Admin Auth
// ---------------------------------------------------------------------------

/// POST /api/v1/admin/login
pub async fn admin_login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let username = std::env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".into());
    let password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin".into());

    if body.username != username || body.password != password {
        return Err(AppError::Unauthorized("Invalid credentials".into()));
    }

    let token = Uuid::new_v4().to_string();
    *state.admin_token.lock().map_err(|e| AppError::Internal(e.to_string()))? = Some(token.clone());

    Ok(Json(json!({"token": token})))
}

/// Middleware to check admin auth token on protected routes.
pub async fn admin_auth(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: middleware::Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    let stored = state
        .admin_token
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?
        .clone();

    match (auth_header, stored) {
        (Some(token), Some(stored_token)) if token == stored_token => Ok(next.run(req).await),
        _ => Err(AppError::Unauthorized("Authentication required".into())),
    }
}

// ---------------------------------------------------------------------------
// Admin Handlers
// ---------------------------------------------------------------------------

/// GET /api/v1/admin/stats/overview
pub async fn admin_overview(
    State(state): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Result<Json<Overview>, AppError> {
    let (from, to) = parse_date_range(&params)?;
    let overview = repository::get_overview(&state.db, from, to).await?;
    Ok(Json(overview))
}

/// GET /api/v1/admin/stats/dau
pub async fn admin_dau(
    State(state): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Result<Json<Vec<DauCount>>, AppError> {
    let (from, to) = parse_date_range(&params)?;
    let rows = repository::get_dau(&state.db, from, to).await?;
    Ok(Json(rows))
}

/// GET /api/v1/admin/stats/pageviews
pub async fn admin_pageviews(
    State(state): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Result<Json<Vec<PvCount>>, AppError> {
    let (from, to) = parse_date_range(&params)?;
    let rows = repository::get_pageviews(&state.db, from, to).await?;
    Ok(Json(rows))
}

/// GET /api/v1/admin/stats/pages
pub async fn admin_page_ranking(
    State(state): State<AppState>,
    Query(params): Query<DateRangeParams>,
) -> Result<Json<Vec<PageRank>>, AppError> {
    let (from, to) = parse_date_range(&params)?;
    let rows = repository::get_page_ranking(&state.db, from, to, 20).await?;
    Ok(Json(rows))
}

/// GET /api/v1/admin/feedback
pub async fn admin_feedback(
    State(state): State<AppState>,
) -> Result<Json<Vec<Feedback>>, AppError> {
    let rows = repository::list_feedback(&state.db).await?;
    Ok(Json(rows))
}

/// GET /api/v1/admin/skills?page=1&per_page=20
pub async fn admin_skills(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Skill>>, AppError> {
    let (page, per_page) = validate_pagination(params.page, params.per_page);
    let result = repository::list_skills(&state.db, page, per_page, None, None, None).await?;
    Ok(Json(result))
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

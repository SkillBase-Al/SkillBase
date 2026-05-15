use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::api::handlers;
use crate::llm::provider::LlmClient;
use sqlx::PgPool;

/// Shared application state available to all request handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub llm_provider: Arc<LlmClient>,
    pub crawl_tx: broadcast::Sender<()>,
}

/// Build the complete axum router with all API routes mounted under `/api/v1`.
pub fn build_router(
    db: PgPool,
    llm_provider: Arc<LlmClient>,
    crawl_tx: broadcast::Sender<()>,
) -> Router {
    let state = AppState {
        db,
        llm_provider,
        crawl_tx,
    };

    Router::new()
        .route("/api/v1/health", get(handlers::health))
        .route("/api/v1/stats", get(handlers::stats))
        .route("/api/v1/skills", get(handlers::list_skills))
        .route("/api/v1/skills/search", get(handlers::search_skills))
        .route("/api/v1/skills/similarity", post(handlers::similarity))
        .route("/api/v1/skills/{id}", get(handlers::get_skill_by_id))
        .route("/api/v1/skills/{id}/similar", get(handlers::get_similar_skills))
        .route("/api/v1/categories", get(handlers::get_categories))
        .route("/api/v1/crawl/trigger", post(handlers::trigger_crawl))
        .route("/api/v1/assess", post(handlers::assess))
        .with_state(state)
}

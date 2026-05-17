use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

use crate::api::handlers;
use crate::llm::provider::LlmClient;
use sqlx::PgPool;

/// Shared application state available to all request handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub llm_provider: Arc<LlmClient>,
    pub crawl_tx: broadcast::Sender<()>,
    pub admin_token: Arc<Mutex<Option<String>>>,
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
        admin_token: Arc::new(Mutex::new(None)),
    };

    let admin_api = Router::new()
        .route("/stats/overview", get(handlers::admin_overview))
        .route("/stats/dau", get(handlers::admin_dau))
        .route("/stats/pageviews", get(handlers::admin_pageviews))
        .route("/stats/pages", get(handlers::admin_page_ranking))
        .route("/feedback", get(handlers::admin_feedback))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            handlers::admin_auth,
        ));

    Router::new()
        .route("/api/v1/health", get(handlers::health))
        .route("/api/v1/stats", get(handlers::stats))
        .route("/api/v1/skills", get(handlers::list_skills))
        .route("/api/v1/skills/search", get(handlers::search_skills))
        .route("/api/v1/skills/similarity", post(handlers::similarity))
        .route("/api/v1/skills/:id", get(handlers::get_skill_by_id))
        .route("/api/v1/skills/:id/similar", get(handlers::get_similar_skills))
        .route("/api/v1/categories", get(handlers::get_categories))
        .route("/api/v1/crawl/trigger", post(handlers::trigger_crawl))
        .route("/api/v1/assess", post(handlers::assess))
        // Telemetry
        .route("/api/v1/telemetry/heartbeat", post(handlers::heartbeat))
        .route("/api/v1/telemetry/pageview", post(handlers::pageview))
        // Feedback
        .route("/api/v1/feedback", post(handlers::submit_feedback))
        // Admin login (no auth required)
        .route("/api/v1/admin/login", post(handlers::admin_login))
        // Admin API (auth required)
        .nest("/api/v1/admin", admin_api)
        // Admin SPA static files
        .nest_service("/admin", ServeDir::new("../admin/dist"))
        .with_state(state)
}

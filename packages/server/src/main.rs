mod api;
mod assessment;
mod crawler;
mod db;
mod embedding;
mod llm;
mod pipeline;

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::api::router;
use crate::crawler::scheduler;
use crate::llm::provider::LlmClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file (ignore error if not found)
    let _ = dotenvy::dotenv();

    // Initialize tracing with env-filter support
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!("Starting SkillBase server...");

    // Database connection pool
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable is required");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    tracing::info!("Connected to database");

    // Run database migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Database migrations completed");

    // Initialize LLM provider
    let llm_client = Arc::new(LlmClient::from_env());
    tracing::info!(
        "LLM provider configured: {:?} / {}",
        llm_client.provider_type(),
        std::env::var("LLM_MODEL").unwrap_or_else(|_| "unknown".to_string())
    );

    // Broadcast channel for triggering crawls
    let (crawl_tx, crawl_rx) = tokio::sync::broadcast::channel::<()>(8);

    // Start the crawler scheduler in the background
    let scheduler_pool = pool.clone();
    let scheduler_llm = llm_client.clone();
    tokio::spawn(async move {
        scheduler::run(scheduler_pool, scheduler_llm, crawl_rx).await;
    });

    tracing::info!("Crawl scheduler started");

    // Build the axum router
    let app = router::build_router(pool, llm_client, crawl_tx)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http());

    // Bind address
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "3007".to_string());
    let addr = format!("{}:{}", host, port);

    tracing::info!("Listening on {}", addr);

    // Start the server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

/// Wait for SIGTERM or SIGINT (Ctrl+C) for graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, shutting down gracefully...");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, shutting down gracefully...");
        }
    }
}

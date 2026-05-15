use std::time::Duration;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use sqlx::PgPool;
use tokio::sync::broadcast;

use crate::crawler;
use crate::llm::provider::LlmClient;
use crate::pipeline::processor;

const BASE_INTERVAL_SECS: u64 = 6 * 3600; // 6 hours
const JITTER_MAX_SECS: u64 = 900; // 15 minutes

/// Run the crawl scheduler loop.
///
/// On first invocation, a full crawl is triggered immediately.
/// Subsequent crawls run every ~6 hours with ±15 min random jitter.
/// The loop can be interrupted early by sending a message on `crawl_rx`.
pub async fn run(
    pool: PgPool,
    llm_client: std::sync::Arc<LlmClient>,
    mut crawl_rx: broadcast::Receiver<()>,
) {
    tracing::info!("Crawl scheduler started — initial crawl will begin immediately");

    loop {
        // Drain any pending trigger messages before crawling
        while crawl_rx.try_recv().is_ok() {}

        tracing::info!("Starting crawl cycle...");
        let raw_skills = crawler::crawl_all().await;

        if let Err(e) = processor::process(&pool, &llm_client, raw_skills).await {
            tracing::error!("Pipeline processing failed: {}", e);
        } else {
            tracing::info!("Crawl cycle completed successfully");
        }

        // Wait for next interval or an early trigger
        let mut rng = StdRng::from_entropy();
        let jitter = rng.gen_range(0..=JITTER_MAX_SECS);
        let interval = Duration::from_secs(BASE_INTERVAL_SECS - JITTER_MAX_SECS / 2 + jitter);

        tracing::info!("Next crawl in ~{:.1} hours", interval.as_secs_f64() / 3600.0);

        let sleep = tokio::time::sleep(interval);
        tokio::pin!(sleep);

        tokio::select! {
            _ = &mut sleep => {
                tracing::info!("Scheduled crawl interval elapsed");
            }
            _ = crawl_rx.recv() => {
                tracing::info!("Crawl triggered early via API signal");
            }
        }
    }
}

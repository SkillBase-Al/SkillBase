use std::sync::Arc;

use sqlx::PgPool;

use crate::crawler::RawSkill;
use crate::db::repository;
use crate::llm::provider::LlmClient;

/// Process a batch of raw skills through the pipeline:
/// 1. License filtering (MIT / Apache-2.0 only)
/// 2. Format validation (basic markdown structure)
/// 3. Basic security check
/// 4. Database upsert (dedup by content hash)
pub async fn process(
    pool: &PgPool,
    _llm_client: &Arc<LlmClient>,
    raw_skills: Vec<RawSkill>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut processed = 0u32;
    let mut skipped_license = 0u32;
    let mut skipped_format = 0u32;
    let mut skipped_security = 0u32;
    let mut deduped = 0u32;

    for skill in raw_skills {
        // 1. License filter
        if !is_license_allowed(&skill.license) {
            skipped_license += 1;
            continue;
        }

        // 2. Format validation
        if !is_valid_format(&skill.content) {
            skipped_format += 1;
            continue;
        }

        // 3. Basic security check
        if !is_safe(&skill.content) {
            skipped_security += 1;
            continue;
        }

        // 4. Upsert to DB (dedup by content hash via ON CONFLICT)
        match repository::upsert_skill(pool, &skill).await {
            Ok(s) => {
                // Check if this was an insert or an update
                if s.created_at == s.updated_at {
                    tracing::debug!("Inserted new skill: {}", s.name);
                } else {
                    tracing::debug!("Updated existing skill: {}", s.name);
                    deduped += 1;
                }
                processed += 1;
            }
            Err(e) => {
                tracing::warn!("Failed to upsert skill '{}': {}", skill.name, e);
            }
        }
    }

    tracing::info!(
        "Pipeline complete: processed={}, skipped_license={}, skipped_format={}, skipped_security={}, deduped={}",
        processed, skipped_license, skipped_format, skipped_security, deduped
    );

    Ok(())
}

/// Only accept MIT or Apache-2.0 licensed skills.
/// Allow unknown licenses (None) since GitHub code search API
/// often doesn't return license data.
fn is_license_allowed(license: &Option<String>) -> bool {
    match license.as_deref() {
        Some(l) => {
            let lower = l.to_lowercase();
            lower == "mit"
                || lower == "apache-2.0"
                || lower == "apache 2.0"
                || lower == "apache2.0"
                || lower == "apache-2"
                || lower == "apache2"
        }
        None => true, // Unknown license — allow through
    }
}

/// Basic markdown structure validation.
fn is_valid_format(content: &str) -> bool {
    if content.len() < 50 {
        return false;
    }
    // Should contain at least one markdown heading
    content.contains("# ") || content.contains("## ") || content.contains("### ")
}

/// Basic security check to reject obviously dangerous content.
fn is_safe(content: &str) -> bool {
    let dangerous = [
        "rm -rf /",
        "rm -rf /*",
        "DROP TABLE ",
        "DROP DATABASE ",
        "eval(",
        "exec(",
        "os.system(",
        "subprocess.call(",
        "Invoke-Expression ",
        "Invoke-Command ",
    ];
    let lower = content.to_lowercase();
    !dangerous.iter().any(|d| lower.contains(d))
}

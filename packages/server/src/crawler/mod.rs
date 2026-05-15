pub mod github_adapter;
pub mod scheduler;
pub mod skillnet_adapter;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A raw skill fetched from a crawler adapter, before pipeline processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSkill {
    pub name: String,
    pub description: Option<String>,
    pub source: String,
    pub source_url: Option<String>,
    pub license: Option<String>,
    pub content: String,
    pub content_hash: String,
}

impl RawSkill {
    pub fn new(
        name: String,
        description: Option<String>,
        source: String,
        source_url: Option<String>,
        license: Option<String>,
        content: String,
    ) -> Self {
        let content_hash = {
            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        Self {
            name,
            description,
            source,
            source_url,
            license,
            content,
            content_hash,
        }
    }
}

/// Run all crawler adapters and collect raw skills.
pub async fn crawl_all() -> Vec<RawSkill> {
    let mut all_skills = Vec::new();

    match github_adapter::fetch_skills().await {
        Ok(skills) => {
            tracing::info!("GitHub adapter returned {} skills", skills.len());
            all_skills.extend(skills);
        }
        Err(e) => {
            tracing::error!("GitHub adapter error: {}", e);
        }
    }

    match skillnet_adapter::fetch_skills().await {
        Ok(skills) => {
            tracing::info!("SkillNet adapter returned {} skills", skills.len());
            all_skills.extend(skills);
        }
        Err(e) => {
            tracing::error!("SkillNet adapter error: {}", e);
        }
    }

    all_skills
}

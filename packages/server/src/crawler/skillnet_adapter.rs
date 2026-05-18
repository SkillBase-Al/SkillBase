use std::time::Duration;

use crate::crawler::RawSkill;
use serde::Deserialize;

const API_BASE: &str = "http://api-skillnet.openkg.cn/v1";
const MAX_PAGES: u32 = 10; // 10 pages × 50 = top 500 skills
const PER_PAGE: u32 = 50;
const MAX_CONCURRENT_FETCHES: usize = 10;

/// Fetch the top 500 skills from SkillNet (sorted by stars descending),
/// then fetch each skill's raw SKILL.md content from GitHub.
pub async fn fetch_skills() -> Result<Vec<RawSkill>, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("skill-manager/0.1")
        .timeout(Duration::from_secs(30))
        .build()?;

    // Step 1: collect all items from SkillNet search (pages 1-N)
    let mut all_items: Vec<SkillNetItem> = Vec::new();

    for page in 1..=MAX_PAGES {
        let url = format!(
            "{}/search?q=&sort_by=stars&sort_order=desc&limit={}&page={}",
            API_BASE, PER_PAGE, page
        );

        match fetch_search_page(&client, &url).await {
            Ok(resp) => {
                let count = resp.data.len();
                tracing::info!(
                    "SkillNet page {}/{}: {} items (total available: {})",
                    page,
                    MAX_PAGES,
                    count,
                    resp.meta.total
                );
                all_items.extend(resp.data);
                if count < PER_PAGE as usize {
                    tracing::info!("SkillNet: no more items, stopping at page {}", page);
                    break;
                }
            }
            Err(e) => {
                tracing::warn!("SkillNet page {} fetch failed: {}", page, e);
                break;
            }
        }
    }

    tracing::info!("SkillNet total items collected: {}", all_items.len());

    // Step 2: fetch raw SKILL.md content for each item (with concurrency limit)
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_FETCHES));
    let mut handles = Vec::new();

    for item in all_items {
        let sem = semaphore.clone();
        let client = client.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            let content = match fetch_skill_content(&client, &item.skill_url).await {
                Some(c) => c,
                None => {
                    tracing::debug!("Skipping {}: failed to fetch content", item.skill_name);
                    return None;
                }
            };

            let mut skill = RawSkill::new(
                format!("skillnet/{}", item.skill_name),
                Some(item.skill_description.clone()),
                "skillnet".to_string(),
                Some(item.skill_url.clone()),
                None, // SkillNet doesn't provide license info
                content,
            );
            // Store SkillNet's star count as the rating for ranking
            skill.rating = Some(item.stars as f64);

            Some(skill)
        }));
    }

    let mut skills = Vec::new();
    for handle in handles {
        if let Some(skill) = handle.await.unwrap_or(None) {
            skills.push(skill);
        }
    }

    tracing::info!("SkillNet adapter: {} skills with content fetched", skills.len());
    Ok(skills)
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    data: Vec<SkillNetItem>,
    meta: SearchMeta,
    success: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SearchMeta {
    query: String,
    mode: String,
    total: u32,
    limit: u32,
    page: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct SkillNetItem {
    skill_name: String,
    skill_description: String,
    author: Option<String>,
    stars: u32,
    skill_url: String,
    category: Option<String>,
    evaluation: SkillNetEvaluation,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct SkillNetEvaluation {
    safety: EvalDimension,
    completeness: EvalDimension,
    executability: EvalDimension,
    cost_awareness: EvalDimension,
    maintainability: EvalDimension,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct EvalDimension {
    level: String,
    reason: Option<String>,
}

async fn fetch_search_page(
    client: &reqwest::Client,
    url: &str,
) -> Result<SearchResponse, Box<dyn std::error::Error + Send + Sync>> {
    let resp = client
        .get(url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(format!("SkillNet API error: HTTP {}", resp.status()).into());
    }

    let data: SearchResponse = resp.json().await?;
    Ok(data)
}

/// Fetch the raw SKILL.md content from a SkillNet skill_url.
/// Converts GitHub blob URLs to raw.githubusercontent.com URLs.
async fn fetch_skill_content(
    client: &reqwest::Client,
    skill_url: &str,
) -> Option<String> {
    // Only handle GitHub URLs
    if !skill_url.starts_with("https://github.com/") {
        // Non-GitHub skills can't be content-fetched by this adapter
        return None;
    }

    // Convert: https://github.com/OWNER/REPO/blob/COMMIT/PATH
    //       -> https://raw.githubusercontent.com/OWNER/REPO/COMMIT/PATH/SKILL.md
    let raw_url = skill_url
        .replace("https://github.com/", "https://raw.githubusercontent.com/")
        .replace("/blob/", "/");

    // Append /SKILL.md if the path doesn't already end with it
    let raw_url = if raw_url.ends_with("/SKILL.md") {
        raw_url
    } else {
        format!("{}/SKILL.md", raw_url.trim_end_matches('/'))
    };

    match client.get(&raw_url).send().await {
        Ok(resp) if resp.status().is_success() => resp.text().await.ok(),
        Ok(resp) => {
            tracing::debug!("Failed to fetch raw content from {}: HTTP {}", raw_url, resp.status());
            None
        }
        Err(e) => {
            tracing::debug!("Failed to fetch raw content from {}: {}", raw_url, e);
            None
        }
    }
}

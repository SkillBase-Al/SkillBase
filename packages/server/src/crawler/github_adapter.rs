use std::time::Duration;

use crate::crawler::RawSkill;
use serde::Deserialize;

const GITHUB_API: &str = "https://api.github.com";
const SEARCH_QUERY: &str = "filename:SKILL.md";
const PER_PAGE: u32 = 100;
const MAX_PAGES: u32 = 2;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubSearchResponse {
    total_count: u32,
    items: Vec<GitHubSearchItem>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubSearchItem {
    name: String,
    path: String,
    html_url: String,
    repository: GitHubRepo,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubRepo {
    full_name: String,
    html_url: String,
    license: Option<GitHubLicense>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubLicense {
    key: Option<String>,
    name: Option<String>,
    spdx_id: Option<String>,
}

/// Fetch skills from GitHub by searching for SKILL.md files with permissive licenses.
pub async fn fetch_skills() -> Result<Vec<RawSkill>, Box<dyn std::error::Error + Send + Sync>> {
    let token = std::env::var("GITHUB_TOKEN")
        .map_err(|_| "GITHUB_TOKEN environment variable not set".to_string())?;

    let client = reqwest::Client::builder()
        .user_agent("skill-manager/0.1")
        .timeout(Duration::from_secs(30))
        .build()?;

    let mut all_items = Vec::new();
    let mut retry_count: u32 = 0;
    let max_retries: u32 = 3;

    for page in 1..=MAX_PAGES {
        let url = format!(
            "{}/search/code?q={}&per_page={}&page={}",
            GITHUB_API, SEARCH_QUERY, PER_PAGE, page
        );

        let search_data = fetch_search_page(&client, &token, &url, &mut retry_count, max_retries)
            .await?;

        if search_data.items.is_empty() {
            tracing::info!("No more GitHub search results on page {}", page);
            break;
        }

        all_items.extend(search_data.items);
        tracing::info!("Fetched page {}/{} of GitHub search results", page, MAX_PAGES);
    }

    tracing::info!("Total GitHub items found: {}", all_items.len());

    let mut skills = Vec::new();
    for item in &all_items {
        match fetch_file_content(&client, &token, &item.repository.full_name, &item.path).await {
            Ok(content) => {
                let license = item
                    .repository
                    .license
                    .as_ref()
                    .and_then(|l| l.spdx_id.clone())
                    .or_else(|| item.repository.license.as_ref().and_then(|l| l.key.clone()));

                let skill = RawSkill::new(
                    format!("{}/{}", item.repository.full_name, item.path),
                    None,
                    "github".to_string(),
                    Some(item.html_url.clone()),
                    license,
                    content,
                );

                skills.push(skill);
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to fetch content for {}: {}",
                    item.repository.full_name,
                    e
                );
            }
        }
    }

    Ok(skills)
}

async fn fetch_search_page(
    client: &reqwest::Client,
    token: &str,
    url: &str,
    retry_count: &mut u32,
    max_retries: u32,
) -> Result<GitHubSearchResponse, Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let resp = client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        let status = resp.status();

        // Rate limiting
        if status == 403 || status == 429 {
            let reset_header = resp
                .headers()
                .get("X-RateLimit-Reset")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());

            if let Some(reset_unix) = reset_header {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if reset_unix > now {
                    let wait = reset_unix - now + 1;
                    tracing::warn!("GitHub API rate limited. Sleeping for {}s", wait);
                    tokio::time::sleep(Duration::from_secs(wait)).await;
                    continue;
                }
            }

            if *retry_count < max_retries {
                let backoff = 2u64.pow(*retry_count) * 5;
                tracing::warn!("GitHub API rate limited. Backing off {}s", backoff);
                tokio::time::sleep(Duration::from_secs(backoff)).await;
                *retry_count += 1;
                continue;
            }

            return Err(format!(
                "GitHub API rate limited after {} retries (HTTP {})",
                max_retries, status
            )
            .into());
        }

        // Server errors with retry
        if status.is_server_error() {
            if *retry_count < max_retries {
                let backoff = 2u64.pow(*retry_count) * 2;
                tracing::warn!(
                    "GitHub API server error {}. Retrying in {}s",
                    status,
                    backoff
                );
                tokio::time::sleep(Duration::from_secs(backoff)).await;
                *retry_count += 1;
                continue;
            }
            return Err(format!("GitHub API server error {} after retries", status).into());
        }

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("GitHub API error ({}): {}", status, body).into());
        }

        *retry_count = 0;
        let data: GitHubSearchResponse = resp.json().await?;
        return Ok(data);
    }
}

/// Fetch the raw content of a file from GitHub.
///
/// Tries raw.githubusercontent.com first for common branch names.
/// Falls back to the GitHub Content API.
async fn fetch_file_content(
    client: &reqwest::Client,
    token: &str,
    repo: &str,
    path: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Try raw content URLs first (no base64 decoding needed)
    let branches = ["main", "master", "develop"];
    for branch in &branches {
        let url = format!("https://raw.githubusercontent.com/{}/{}/{}", repo, branch, path);
        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if resp.status().is_success() {
            return Ok(resp.text().await?);
        }
    }

    // Fallback: use the GitHub Content API (base64 encoded)
    let url = format!("{}/repos/{}/contents/{}", GITHUB_API, repo, path);
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(format!(
            "Failed to fetch content for {}/{}: HTTP {}",
            repo,
            path,
            resp.status()
        )
        .into());
    }

    #[derive(Deserialize)]
    struct ContentResponse {
        content: Option<String>,
    }

    let data: ContentResponse = resp.json().await?;
    match data.content {
        Some(encoded) => {
            // GitHub base64 content has newlines; strip and decode
            let clean = encoded.replace('\n', "").replace('\r', "");
            decode_base64(&clean)
        }
        None => Err(format!("Empty content response for {}/{}", repo, path).into()),
    }
}

/// Minimal base64 decoder. Avoids pulling in the `base64` crate.
fn decode_base64(input: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut bytes = Vec::with_capacity(input.len() * 3 / 4);
    let input = input.trim_end_matches('=');

    for chunk in input.as_bytes().chunks(4) {
        let mut buf: u32 = 0;
        for (i, &b) in chunk.iter().enumerate() {
            let val = CHARS
                .iter()
                .position(|&c| c == b)
                .ok_or_else(|| format!("Invalid base64 character: {}", b as char))? as u32;
            buf |= val << (6 * (3 - i));
        }

        let n = chunk.len();
        bytes.push((buf >> 16) as u8);
        if n >= 3 {
            bytes.push((buf >> 8) as u8);
        }
        if n >= 4 {
            bytes.push(buf as u8);
        }
    }

    String::from_utf8(bytes).map_err(|e| e.into())
}

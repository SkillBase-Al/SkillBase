use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);
const CACHE_TTL: Duration = Duration::from_secs(600);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSkill {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub source: String,
    pub source_url: Option<String>,
    pub license: Option<String>,
    pub author: Option<String>,
    pub rating: Option<f64>,
    pub install_count: Option<i32>,
    #[serde(default)]
    pub categories: Vec<String>,
    pub safety_level: Option<String>,
    pub format_score: Option<i32>,
    pub quality_score: Option<i32>,
    pub skill_md_content: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityRequest {
    pub descriptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityResponse {
    pub similarities: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStats {
    pub total_skills: i32,
    pub sources: Vec<SourceCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCount {
    pub source: String,
    pub count: i32,
}

pub struct MarketClient {
    base_url: String,
    client: reqwest::Client,
}

impl MarketClient {
    pub fn new(base_url: &str, proxy_url: Option<&str>) -> Self {
        let mut builder = reqwest::Client::builder()
            .timeout(DEFAULT_TIMEOUT);

        if let Some(proxy) = proxy_url {
            if !proxy.is_empty() {
                if let Ok(p) = reqwest::Proxy::all(proxy) {
                    // Bypass proxy for localhost and loopback addresses
                    let p = p.no_proxy(reqwest::NoProxy::from_string("localhost,127.0.0.1,::1"));
                    builder = builder.proxy(p);
                }
            }
        }

        let client = builder.build().unwrap_or_default();

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    pub async fn search_skills(
        &self,
        query: &str,
        category: Option<&str>,
        page: i32,
        per_page: i32,
    ) -> Result<PaginatedResponse<MarketSkill>, String> {
        let mut url = format!("{}/api/v1/skills/search?q={}&page={}&per_page={}",
            self.base_url, urlencode(query), page, per_page);
        if let Some(cat) = category {
            url.push_str(&format!("&category={}", urlencode(cat)));
        }

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Search request failed: {}", e))?;

        resp.json::<PaginatedResponse<MarketSkill>>()
            .await
            .map_err(|e| format!("Search parse failed: {}", e))
    }

    pub async fn list_skills(
        &self,
        page: i32,
        per_page: i32,
        category: Option<&str>,
        sort: Option<&str>,
    ) -> Result<PaginatedResponse<MarketSkill>, String> {
        let mut url = format!("{}/api/v1/skills?page={}&per_page={}", self.base_url, page, per_page);
        if let Some(cat) = category {
            url.push_str(&format!("&category={}", urlencode(cat)));
        }
        if let Some(s) = sort {
            url.push_str(&format!("&sort={}", s));
        }

        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| format!("List request failed: {}", e))?;

        resp.json::<PaginatedResponse<MarketSkill>>()
            .await
            .map_err(|e| format!("List parse failed: {}", e))
    }

    pub async fn get_skill_detail(&self, id: &str) -> Result<MarketSkill, String> {
        let url = format!("{}/api/v1/skills/{}", self.base_url, id);
        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Detail request failed: {}", e))?;

        resp.json::<MarketSkill>()
            .await
            .map_err(|e| format!("Detail parse failed: {}", e))
    }

    pub async fn get_categories(&self) -> Result<Vec<Category>, String> {
        let url = format!("{}/api/v1/categories", self.base_url);
        let resp = self.client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Categories request failed: {}", e))?;

        resp.json::<Vec<Category>>()
            .await
            .map_err(|e| format!("Categories parse failed: {}", e))
    }

    pub async fn compute_similarity(&self, descriptions: Vec<String>) -> Result<SimilarityResponse, String> {
        let url = format!("{}/api/v1/skills/similarity", self.base_url);
        let req = SimilarityRequest { descriptions };

        let resp = self.client.post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("Similarity request failed: {}", e))?;

        resp.json::<SimilarityResponse>()
            .await
            .map_err(|e| format!("Similarity parse failed: {}", e))
    }

    pub async fn assess_skill(&self, skill_content: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/api/v1/assess", self.base_url);
        let body = serde_json::json!({ "skill_content": skill_content });

        let resp = self.client.post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Assess request failed: {}", e))?;

        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| format!("Assess parse failed: {}", e))
    }
}

fn urlencode(s: &str) -> String {
    urlencoding(s)
}

fn urlencoding(s: &str) -> String {
    s.chars().map(|c| match c {
        'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
        ' ' => "+".to_string(),
        other => format!("%{:02X}", other as u8),
    }).collect::<Vec<_>>().join("")
}

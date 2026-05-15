use crate::api_client::market_client::{MarketClient, MarketSkill, Category, PaginatedResponse};

/// Search the marketplace for skills
#[tauri::command]
pub async fn search_market(
    query: String,
    category: Option<String>,
    page: Option<i32>,
    per_page: Option<i32>,
) -> Result<PaginatedResponse<MarketSkill>, String> {
    let base_url = std::env::var("SKILLBASE_API_URL").unwrap_or_else(|_| "http://localhost:3000".into());
    let client = MarketClient::new(&base_url);

    client.search_skills(
        &query,
        category.as_deref(),
        page.unwrap_or(1),
        per_page.unwrap_or(20),
    ).await
}

/// Get skill detail from marketplace
#[tauri::command]
pub async fn get_skill_detail(skill_id: String) -> Result<MarketSkill, String> {
    let base_url = std::env::var("SKILLBASE_API_URL").unwrap_or_else(|_| "http://localhost:3000".into());
    let client = MarketClient::new(&base_url);

    client.get_skill_detail(&skill_id).await
}

/// Get marketplace categories
#[tauri::command]
pub async fn get_categories() -> Result<Vec<Category>, String> {
    let base_url = std::env::var("SKILLBASE_API_URL").unwrap_or_else(|_| "http://localhost:3000".into());
    let client = MarketClient::new(&base_url);

    client.get_categories().await
}

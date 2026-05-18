use tauri::State;
use crate::api_client::market_client::{MarketClient, MarketSkill, Category, PaginatedResponse};
use crate::db::repository::{self, DbConn};

fn get_proxy_url(conn: &DbConn) -> Option<String> {
    repository::get_setting(conn, "proxy_url").ok().flatten()
}

fn get_server_url(_conn: &DbConn) -> String {
    // Server URL is fixed at build/deploy time — not user-configurable
    std::env::var("SKILLBASE_SERVER_URL")
        .unwrap_or_else(|_| "http://skills.yy-crow.com".into())
}

/// Search the marketplace for skills.
/// When the query is empty, falls back to the list endpoint to return all skills.
#[tauri::command]
pub async fn search_market(
    conn: State<'_, DbConn>,
    query: String,
    category: Option<String>,
    page: Option<i32>,
    per_page: Option<i32>,
) -> Result<PaginatedResponse<MarketSkill>, String> {
    let base_url = get_server_url(&conn);
    let proxy_url = get_proxy_url(&conn);
    let client = MarketClient::new(&base_url, proxy_url.as_deref());

    if query.trim().is_empty() {
        // No search query — use the list endpoint which returns all skills
        client.list_skills(
            page.unwrap_or(1),
            per_page.unwrap_or(20),
            category.as_deref(),
            None,
        ).await
    } else {
        client.search_skills(
            &query,
            category.as_deref(),
            page.unwrap_or(1),
            per_page.unwrap_or(20),
        ).await
    }
}

/// Get skill detail from marketplace
#[tauri::command]
pub async fn get_skill_detail(
    conn: State<'_, DbConn>,
    skill_id: String,
) -> Result<MarketSkill, String> {
    let base_url = get_server_url(&conn);
    let proxy_url = get_proxy_url(&conn);
    let client = MarketClient::new(&base_url, proxy_url.as_deref());

    client.get_skill_detail(&skill_id).await
}

/// Get marketplace categories
#[tauri::command]
pub async fn get_categories(
    conn: State<'_, DbConn>,
) -> Result<Vec<Category>, String> {
    let base_url = get_server_url(&conn);
    let proxy_url = get_proxy_url(&conn);
    let client = MarketClient::new(&base_url, proxy_url.as_deref());

    client.get_categories().await
}

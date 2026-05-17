use tauri::State;
use crate::db::models::AppSettings;
use crate::db::repository::{self, DbConn};

#[tauri::command]
pub fn get_settings(conn: State<DbConn>) -> Result<AppSettings, String> {
    Ok(AppSettings {
        theme: repository::get_setting(&conn, "theme")
            .ok().flatten().unwrap_or_else(|| "system".into()),
        server_url: repository::get_setting(&conn, "server_url")
            .ok().flatten().unwrap_or_default(),
        proxy_url: repository::get_setting(&conn, "proxy_url")
            .ok().flatten().unwrap_or_default(),
        auto_scan: repository::get_setting(&conn, "auto_scan")
            .ok().flatten().map(|v| v == "true").unwrap_or(true),
        auto_assess: repository::get_setting(&conn, "auto_assess")
            .ok().flatten().map(|v| v == "true").unwrap_or(false),
        scan_paths: repository::get_setting(&conn, "scan_paths")
            .ok().flatten()
            .map(|v| serde_json::from_str(&v).unwrap_or_default())
            .unwrap_or_default(),
        sidebar_collapsed: repository::get_setting(&conn, "sidebar_collapsed")
            .ok().flatten().map(|v| v == "true").unwrap_or(false),
        first_run_complete: repository::get_setting(&conn, "first_run_complete")
            .ok().flatten().map(|v| v == "true").unwrap_or(false),
        security_check_enabled: repository::get_setting(&conn, "security_check_enabled")
            .ok().flatten().map(|v| v == "true").unwrap_or(true),
        crawl_repos: repository::get_setting(&conn, "crawl_repos")
            .ok().flatten()
            .map(|v| serde_json::from_str(&v).unwrap_or_default())
            .unwrap_or_else(|| vec!["anthropics/skills".to_string()]),
    })
}

#[tauri::command]
pub fn update_settings(conn: State<DbConn>, settings: AppSettings) -> Result<AppSettings, String> {
    // Store all settings to DB
    repository::set_setting(&conn, "theme", &settings.theme)?;
    repository::set_setting(&conn, "server_url", &settings.server_url)?;
    repository::set_setting(&conn, "proxy_url", &settings.proxy_url)?;
    repository::set_setting(&conn, "auto_scan", if settings.auto_scan { "true" } else { "false" })?;
    repository::set_setting(&conn, "auto_assess", if settings.auto_assess { "true" } else { "false" })?;
    repository::set_setting(&conn, "sidebar_collapsed", if settings.sidebar_collapsed { "true" } else { "false" })?;
    repository::set_setting(&conn, "first_run_complete", if settings.first_run_complete { "true" } else { "false" })?;
    if !settings.scan_paths.is_empty() {
        repository::set_setting(&conn, "scan_paths", &serde_json::to_string(&settings.scan_paths).unwrap_or_default())?;
    }
    repository::set_setting(&conn, "security_check_enabled", if settings.security_check_enabled { "true" } else { "false" })?;
    if !settings.crawl_repos.is_empty() {
        repository::set_setting(&conn, "crawl_repos", &serde_json::to_string(&settings.crawl_repos).unwrap_or_default())?;
    }

    Ok(settings)
}

#[tauri::command]
pub fn get_server_url(conn: State<DbConn>) -> Result<String, String> {
    Ok(repository::get_setting(&conn, "server_url")
        .ok().flatten()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "http://localhost:3007".into()))
}

#[tauri::command]
pub fn check_first_run(conn: State<DbConn>) -> Result<bool, String> {
    let agents = repository::get_all_agents(&conn)?;
    let skills = repository::get_all_skills(&conn)?;
    Ok(agents.is_empty() && skills.is_empty())
}

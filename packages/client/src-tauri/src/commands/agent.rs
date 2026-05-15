use tauri::State;
use uuid::Uuid;
use crate::db::models::AgentConfig;
use crate::db::repository::{self, DbConn};

#[tauri::command]
pub fn get_agents(conn: State<DbConn>) -> Result<Vec<AgentConfig>, String> {
    repository::get_all_agents(&conn)
}

#[tauri::command]
pub fn add_agent(conn: State<DbConn>, name: String, agent_type: String, base_path: String) -> Result<AgentConfig, String> {
    let agent = AgentConfig {
        id: Uuid::new_v4().to_string(),
        name,
        agent_type,
        base_path,
        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: None,
    };

    repository::insert_agent(&conn, &agent)?;
    Ok(agent)
}

#[tauri::command]
pub fn update_agent(conn: State<DbConn>, id: String, name: String, agent_type: String, base_path: String) -> Result<(), String> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let agent = AgentConfig { id, name, agent_type, base_path, created_at: String::new(), updated_at: Some(now) };
    repository::update_agent(&conn, &agent)
}

#[tauri::command]
pub fn delete_agent(conn: State<DbConn>, id: String) -> Result<(), String> {
    repository::delete_agent(&conn, &id)
}

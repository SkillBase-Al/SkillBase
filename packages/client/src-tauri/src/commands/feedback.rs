use tauri::State;
use crate::db::models::Feedback;
use crate::db::repository::{self, DbConn};

#[tauri::command]
pub fn submit_feedback(
    conn: State<DbConn>,
    title: String,
    description: String,
) -> Result<(), String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let fb = Feedback {
        id,
        title,
        description,
        created_at: now,
    };
    repository::insert_feedback(&conn, &fb)?;

    Ok(())
}

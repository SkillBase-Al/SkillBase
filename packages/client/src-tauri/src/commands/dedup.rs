use tauri::State;
use uuid::Uuid;
use crate::db::models::DedupGroup;
use crate::db::repository::{self, DbConn};

#[tauri::command]
pub fn run_dedup(conn: State<DbConn>) -> Result<Vec<DedupGroup>, String> {
    let skills = repository::get_all_skills(&conn)?;
    repository::clear_dedup_groups(&conn)?;

    let groups = crate::dedup::local_dedup::find_duplicates(&skills);

    let mut result = Vec::new();

    for (similarity, level, group_skills) in &groups {
        let group_id = Uuid::new_v4().to_string();
        repository::insert_dedup_group(&conn, &group_id, *similarity, level)?;

        for skill in group_skills {
            repository::add_skill_to_dedup_group(&conn, &group_id, &skill.id)?;
        }

        result.push(DedupGroup {
            id: group_id,
            similarity: *similarity,
            level: level.clone(),
            skills: group_skills.clone(),
        });
    }

    // Sort by similarity descending
    result.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));

    Ok(result)
}

#[tauri::command]
pub fn get_dedup_groups(conn: State<DbConn>) -> Result<Vec<DedupGroup>, String> {
    let raw = repository::get_dedup_groups_with_skills(&conn)?;

    let groups: Vec<DedupGroup> = raw.into_iter()
        .map(|(id, sim, level, skills)| DedupGroup { id, similarity: sim, level, skills })
        .collect();

    Ok(groups)
}

#[tauri::command]
pub fn delete_skill_from_group(conn: State<DbConn>, skill_id: String) -> Result<(), String> {
    repository::delete_skill(&conn, &skill_id)?;
    Ok(())
}

use std::fs;
use std::path::Path;
use sha2::{Sha256, Digest};
use tauri::State;
use crate::db::models::{InstalledSkill, ConflictCandidate, SkillConflict};
use crate::db::repository::{self, DbConn};

#[tauri::command]
pub fn get_installed_skills(conn: State<DbConn>) -> Result<Vec<InstalledSkill>, String> {
    // Deduplicate by name (keep the oldest entry for each unique name)
    dedup_skills(&conn)?;
    // Scan all agent paths first to pick up any new skills
    scan_agent_paths(&conn)?;
    repository::get_all_skills(&conn)
}

/// Remove duplicate skills by name, keeping only the oldest entry
fn dedup_skills(conn: &DbConn) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute_batch(
        "DELETE FROM installed_skills WHERE id NOT IN (
            SELECT MIN(id) FROM installed_skills GROUP BY name
        )"
    ).map_err(|e| e.to_string())?;
    Ok(())
}

/// Scan all agent-configured paths and upsert discovered skills into the DB.
/// When the same skill name is found with DIFFERENT content, a conflict is
/// recorded instead of auto-merging.
fn scan_agent_paths(conn: &DbConn) -> Result<(), String> {
    let agents = repository::get_all_agents(conn)?;
    for agent in &agents {
        let expanded_paths = crate::utils::paths::split_base_paths(&agent.base_path);

        for expanded in expanded_paths {
            if !expanded.exists() {
                continue;
            }

            if let Ok(found) = crate::scanner::skill_scanner::scan_directory(&expanded) {
                let existing = repository::get_all_skills(conn)?;
                for s in &found {
                    let parent_name = s.path.parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");

                    let content = match std::fs::read_to_string(&s.path) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    let (name, description, version) = parse_metadata(&content, parent_name);

                    // Dedup by the actual name from frontmatter
                    if let Some(skill) = existing.iter().find(|sk| sk.name == name) {
                        // Read existing skill content to compare
                        let existing_path = Path::new(&skill.skill_dir).join("SKILL.md");
                        let existing_content = match fs::read_to_string(&existing_path) {
                            Ok(c) => c,
                            Err(_) => {
                                // Stored path is stale — update skill_dir to the current location
                                let new_skill_dir = s.path.parent().unwrap().to_string_lossy().to_string();
                                let mut updated = skill.clone();
                                updated.skill_dir = new_skill_dir;
                                let _ = repository::insert_skill(conn, &updated);

                                // Now try reading from the new location
                                if let Ok(c) = fs::read_to_string(&s.path) {
                                    if c == content {
                                        repository::add_mapping(conn, &skill.id, &agent.id)?;
                                        continue;
                                    }
                                    // Different content — will fall through to conflict handling
                                }
                                repository::add_mapping(conn, &skill.id, &agent.id)?;
                                continue;
                            }
                        };

                        if content == existing_content {
                            // Same content — normal merge
                            repository::add_mapping(conn, &skill.id, &agent.id)?;
                            continue;
                        }

                        // Different content — record conflict, do NOT auto-merge
                        let existing_agent_ids =
                            repository::get_mappings_for_skill(conn, &skill.id)?;
                        let agents_list = repository::get_all_agents(conn)?;
                        let mut candidates: Vec<ConflictCandidate> = Vec::new();

                        for ea_id in &existing_agent_ids {
                            if let Some(ea) = agents_list.iter().find(|a| a.id == *ea_id) {
                                candidates.push(ConflictCandidate {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    skill_id: Some(skill.id.clone()),
                                    skill_dir: skill.skill_dir.clone(),
                                    agent_id: ea.id.clone(),
                                    agent_name: ea.name.clone(),
                                    agent_type: ea.agent_type.clone(),
                                    version: skill.version.clone(),
                                    content_hash: hash_content(&existing_content),
                                });
                            }
                        }

                        let new_skill_dir =
                            s.path.parent().unwrap().to_string_lossy().to_string();
                        candidates.push(ConflictCandidate {
                            id: uuid::Uuid::new_v4().to_string(),
                            skill_id: None,
                            skill_dir: new_skill_dir,
                            agent_id: agent.id.clone(),
                            agent_name: agent.name.clone(),
                            agent_type: agent.agent_type.clone(),
                            version,
                            content_hash: hash_content(&content),
                        });

                        let existing_conflicts = repository::get_unresolved_conflicts(conn)?;
                        if let Some(ec) =
                            existing_conflicts.iter().find(|c| c.skill_name == name)
                        {
                            let mut updated = ec.clone();
                            let new_candidate = candidates.last().unwrap();
                            if !updated
                                .candidates
                                .iter()
                                .any(|c| c.agent_id == agent.id)
                            {
                                updated.candidates.push(new_candidate.clone());
                                repository::insert_conflict(conn, &updated)?;
                            }
                        } else {
                            let conflict = SkillConflict {
                                id: uuid::Uuid::new_v4().to_string(),
                                skill_name: name.clone(),
                                candidates,
                                resolved: false,
                                kept_candidate_id: None,
                                created_at: chrono::Utc::now()
                                    .format("%Y-%m-%d %H:%M:%S")
                                    .to_string(),
                            };
                            repository::insert_conflict(conn, &conflict)?;
                        }
                        continue;
                    }

                    // No existing skill — insert fresh
                    let skill_dir = s.path.parent().unwrap().to_string_lossy().to_string();
                    let skill = InstalledSkill {
                        id: uuid::Uuid::new_v4().to_string(),
                        name,
                        description: description.clone(),
                        source: "local".into(),
                        source_url: None,
                        version: version.clone(),
                        license: None,
                        author: None,
                        skill_dir,
                        enabled: true,
                        installed_at: chrono::Utc::now()
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string(),
                        updated_at: None,
                        format_score: None,
                        safety_level: None,
                        quality_score: None,
                        tags: Vec::new(),
                        has_security_issues: false,
                        agent_count: 0,
                        agent_ids: Vec::new(),
                    };
                    repository::insert_skill(conn, &skill)?;
                    repository::add_mapping(conn, &skill.id, &agent.id)?;
                }
            }
        }
    }
    Ok(())
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ── Conflict commands ──────────────────────────────────────────────

#[tauri::command]
pub fn get_skill_conflicts(conn: State<DbConn>) -> Result<Vec<SkillConflict>, String> {
    repository::get_unresolved_conflicts(&conn)
}

#[tauri::command]
pub fn resolve_skill_conflict(
    conn: State<DbConn>,
    conflict_id: String,
    kept_candidate_id: String,
) -> Result<(), String> {
    let conflicts = repository::get_unresolved_conflicts(&conn)?;
    let conflict = conflicts
        .iter()
        .find(|c| c.id == conflict_id)
        .ok_or_else(|| "Conflict not found".to_string())?;

    let kept = conflict
        .candidates
        .iter()
        .find(|c| c.id == kept_candidate_id)
        .ok_or_else(|| "Candidate not found".to_string())?;

    // Read the kept candidate's SKILL.md content
    let kept_skill_path = Path::new(&kept.skill_dir).join("SKILL.md");
    let kept_content =
        fs::read_to_string(&kept_skill_path).map_err(|e| format!("Read error: {}", e))?;

    // Collect all agent IDs involved in this conflict
    let mut all_agent_ids: Vec<String> = conflict
        .candidates
        .iter()
        .map(|c| c.agent_id.clone())
        .collect();
    all_agent_ids.sort();
    all_agent_ids.dedup();

    // If the kept candidate has an existing skill_id, use it; otherwise create
    let kept_skill_id = if let Some(sid) = &kept.skill_id {
        // Update existing skill record with the kept content
        if let Ok(Some(mut existing_skill)) = repository::get_skill_by_id(&conn, sid) {
            existing_skill.skill_dir = kept.skill_dir.clone();
            existing_skill.version = kept.version.clone();
            repository::insert_skill(&conn, &existing_skill)?;
            sid.clone()
        } else {
            // Skill record was deleted — create new one
            let id = uuid::Uuid::new_v4().to_string();
            let skill = InstalledSkill {
                id: id.clone(),
                name: conflict.skill_name.clone(),
                description: String::new(),
                source: "local".into(),
                source_url: None,
                version: kept.version.clone(),
                license: None,
                author: None,
                skill_dir: kept.skill_dir.clone(),
                enabled: true,
                installed_at: chrono::Utc::now()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                updated_at: None,
                format_score: None,
                safety_level: None,
                quality_score: None,
                tags: Vec::new(),
                has_security_issues: false,
                agent_count: 0,
                agent_ids: Vec::new(),
            };
            repository::insert_skill(&conn, &skill)?;
            id
        }
    } else {
        // New candidate without a skill_id — create skill record
        let id = uuid::Uuid::new_v4().to_string();
        let skill = InstalledSkill {
            id: id.clone(),
            name: conflict.skill_name.clone(),
            description: String::new(),
            source: "local".into(),
            source_url: None,
            version: kept.version.clone(),
            license: None,
            author: None,
            skill_dir: kept.skill_dir.clone(),
            enabled: true,
            installed_at: chrono::Utc::now()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            updated_at: None,
            format_score: None,
            safety_level: None,
            quality_score: None,
            tags: Vec::new(),
            has_security_issues: false,
            agent_count: 0,
            agent_ids: Vec::new(),
        };
        repository::insert_skill(&conn, &skill)?;
        id
    };

    // Install kept content to ALL agents involved, replacing old versions
    let kept_skill_obj = repository::get_skill_by_id(&conn, &kept_skill_id)?
        .ok_or_else(|| "Failed to load kept skill".to_string())?;

    // Clean mappings from old (non-kept) skill records
    for candidate in &conflict.candidates {
        if let Some(sid) = &candidate.skill_id {
            if sid != &kept_skill_id {
                let _ = repository::remove_mappings_for_skill(&conn, sid);
                let _ = repository::delete_skill(&conn, sid);
            }
        }
    }

    // Install kept content to every involved agent
    for agent_id in &all_agent_ids {
        let _ = repository::add_mapping(&conn, &kept_skill_id, agent_id);
    }

    crate::installer::agent_installer::install_to_agents(
        &conn, &kept_skill_obj, &kept_content, &all_agent_ids,
    )?;

    // Mark conflict resolved
    repository::resolve_conflict(&conn, &conflict_id, &kept_candidate_id)?;

    Ok(())
}

// ── Existing commands ──────────────────────────────────────────────

#[tauri::command]
pub fn install_skill(
    conn: State<DbConn>,
    name: String,
    description: String,
    source: String,
    source_url: Option<String>,
    version: Option<String>,
    license: Option<String>,
    author: Option<String>,
    skill_content: String,
    agent_ids: Vec<String>,
) -> Result<InstalledSkill, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let skills_dir = crate::utils::paths::skills_dir();
    let skill_dir = skills_dir.join(&name);

    // Create skill directory
    fs::create_dir_all(&skill_dir)
        .map_err(|e| format!("Failed to create skill dir: {}", e))?;

    // Write SKILL.md
    let skill_md_path = skill_dir.join("SKILL.md");
    fs::write(&skill_md_path, &skill_content)
        .map_err(|e| format!("Failed to write SKILL.md: {}", e))?;

    let skill = InstalledSkill {
        id: id.clone(),
        name,
        description,
        source,
        source_url,
        version,
        license,
        author,
        skill_dir: skill_dir.to_string_lossy().to_string(),
        enabled: true,
        installed_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        updated_at: None,
        format_score: None,
        safety_level: None,
        quality_score: None,
        tags: Vec::new(),
        has_security_issues: false,
        agent_count: 0,
        agent_ids: Vec::new(),
    };

    repository::insert_skill(&conn, &skill)?;

    // Install to agents
    if !agent_ids.is_empty() {
        crate::installer::agent_installer::install_to_agents(
            &conn, &skill, &skill_content, &agent_ids,
        )?;
    }

    Ok(skill)
}

#[tauri::command]
pub fn uninstall_skill(conn: State<DbConn>, id: String) -> Result<(), String> {
    let skill = repository::get_skill_by_id(&conn, &id)?
        .ok_or_else(|| "Skill not found".to_string())?;

    // Remove from agents
    crate::installer::agent_installer::uninstall_from_agents(&conn, &skill)?;

    // Remove skill directory
    let skill_dir = Path::new(&skill.skill_dir);
    if skill_dir.exists() {
        fs::remove_dir_all(skill_dir)
            .map_err(|e| format!("Failed to remove skill dir: {}", e))?;
    }

    repository::delete_skill(&conn, &id)?;
    Ok(())
}

#[tauri::command]
pub fn update_skill(
    conn: State<DbConn>,
    id: String,
    skill_content: String,
) -> Result<(), String> {
    let skill = repository::get_skill_by_id(&conn, &id)?
        .ok_or_else(|| "Skill not found".to_string())?;

    // Update local SKILL.md
    let skill_md_path = Path::new(&skill.skill_dir).join("SKILL.md");
    fs::write(&skill_md_path, &skill_content)
        .map_err(|e| format!("Failed to update SKILL.md: {}", e))?;

    // Update in all agent directories
    crate::installer::agent_installer::update_in_agents(&conn, &skill, &skill_content)?;

    Ok(())
}

#[tauri::command]
pub fn apply_skill_to_agents(
    conn: State<DbConn>,
    skill_id: String,
    agent_ids: Vec<String>,
) -> Result<(), String> {
    let skill = repository::get_skill_by_id(&conn, &skill_id)?
        .ok_or_else(|| "Skill not found".to_string())?;

    let skill_md_path = Path::new(&skill.skill_dir).join("SKILL.md");
    let skill_content =
        fs::read_to_string(&skill_md_path).map_err(|e| format!("Failed to read SKILL.md: {}", e))?;

    crate::installer::agent_installer::install_to_agents(
        &conn, &skill, &skill_content, &agent_ids,
    )?;

    Ok(())
}

#[tauri::command]
pub fn remove_skill_from_agents(
    conn: State<DbConn>,
    skill_id: String,
    agent_ids: Vec<String>,
) -> Result<(), String> {
    let skill = repository::get_skill_by_id(&conn, &skill_id)?
        .ok_or_else(|| "Skill not found".to_string())?;

    crate::installer::agent_installer::uninstall_from_agents_by_ids(
        &conn, &skill, &agent_ids,
    )?;

    Ok(())
}

#[tauri::command]
pub fn toggle_skill_enabled(conn: State<DbConn>, id: String, enabled: bool) -> Result<(), String> {
    let skill = repository::get_skill_by_id(&conn, &id)?
        .ok_or_else(|| "Skill not found".to_string())?;

    crate::installer::agent_installer::toggle_in_agents(&conn, &skill, enabled)
}

#[tauri::command]
pub fn get_skill_content(conn: State<DbConn>, skill_id: String) -> Result<String, String> {
    let skill = repository::get_skill_by_id(&conn, &skill_id)?
        .ok_or_else(|| "Skill not found".to_string())?;
    let skill_md_path = Path::new(&skill.skill_dir).join("SKILL.md");
    fs::read_to_string(&skill_md_path)
        .map_err(|e| format!("Failed to read SKILL.md: {}", e))
}

#[tauri::command]
pub fn scan_local_skills(conn: State<DbConn>, paths: Vec<String>) -> Result<Vec<InstalledSkill>, String> {
    // Collect all scanned skills (deduplicated by path)
    let mut all_scanned = crate::scanner::skill_scanner::scan_default()
        .unwrap_or_default();

    // Also scan each provided path (expand ~ to $HOME)
    for path_str in &paths {
        let expanded = if path_str.starts_with("~/") {
            if let Some(home) = std::env::var("HOME").ok() {
                std::path::PathBuf::from(home).join(&path_str[2..])
            } else {
                std::path::PathBuf::from(path_str)
            }
        } else {
            std::path::PathBuf::from(path_str)
        };
        if let Ok(found) = crate::scanner::skill_scanner::scan_directory(&expanded) {
            for s in found {
                if !all_scanned.iter().any(|existing| existing.path == s.path) {
                    all_scanned.push(s);
                }
            }
        }
    }

    // For each scanned SKILL.md, parse and upsert
    for s in &all_scanned {
        let content = fs::read_to_string(&s.path)
            .map_err(|e| format!("Read error: {}", e))?;

        let parent_name = s.path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let (name, description, version) = parse_metadata(&content, parent_name);

        let existing = repository::get_all_skills(&conn)?;
        // Dedup by the actual name from frontmatter
        let found = existing.iter().find(|sk| sk.name == name);

        if found.is_none() {
            let id = uuid::Uuid::new_v4().to_string();
            let skill = InstalledSkill {
                id,
                name,
                description,
                source: "local".into(),
                source_url: None,
                version,
                license: None,
                author: None,
                skill_dir: s.path.parent().unwrap().to_string_lossy().to_string(),
                enabled: true,
                installed_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: None,
                format_score: None,
                safety_level: None,
                quality_score: None,
                tags: Vec::new(),
                has_security_issues: false,
                agent_count: 0,
                agent_ids: Vec::new(),
            };
            repository::insert_skill(&conn, &skill)?;
        } else {
            // Update existing skill's skill_dir in case the file was moved
            let current_skill_dir = s.path.parent().unwrap().to_string_lossy().to_string();
            if found.unwrap().skill_dir != current_skill_dir {
                let mut updated = found.unwrap().clone();
                updated.skill_dir = current_skill_dir;
                repository::insert_skill(&conn, &updated)?;
            }
        }
    }

    repository::get_all_skills(&conn)
}

#[tauri::command]
pub fn import_skills(
    conn: State<DbConn>,
    paths: Vec<String>,
) -> Result<Vec<InstalledSkill>, String> {
    let skills_dir = crate::utils::paths::skills_dir();

    for path_str in &paths {
        let src = Path::new(path_str);
        if !src.exists() || !src.is_dir() {
            continue;
        }

        // Copy skill directory to ~/.skillbase/skills/
        let dir_name = src.file_name().unwrap_or_default();
        let dest = skills_dir.join(dir_name);

        if !dest.exists() {
            copy_dir_recursive(src, &dest)
                .map_err(|e| format!("Failed to copy skill: {}", e))?;
        }
    }

    scan_local_skills(conn, Vec::new())
}

/// Parse name, description, and version from SKILL.md frontmatter
fn parse_metadata(content: &str, fallback_name: &str) -> (String, String, Option<String>) {
    let result = crate::checker::format_checker::assess(content);
    match result.metadata {
        Some(yaml) => {
            let map = yaml.as_mapping().unwrap();
            let name = map
                .get(&serde_yaml::Value::String("name".into()))
                .and_then(|v| v.as_str())
                .unwrap_or(fallback_name)
                .to_string();
            let desc = map
                .get(&serde_yaml::Value::String("description".into()))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let version = map
                .get(&serde_yaml::Value::String("version".into()))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            (name, desc, version)
        }
        None => (fallback_name.to_string(), String::new(), None),
    }
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dest.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(&entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

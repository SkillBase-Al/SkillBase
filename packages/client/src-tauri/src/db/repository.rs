use rusqlite::{Connection, params};
use std::path::Path;
use std::sync::Mutex;
use crate::db::models::*;

pub type DbConn = Mutex<Connection>;

pub fn init_db(app_dir: &Path) -> Result<DbConn, String> {
    let db_path = crate::utils::paths::index_db_path();
    let conn = Connection::open(&db_path).map_err(|e| format!("DB open error: {}", e))?;

    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(|e| format!("PRAGMA error: {}", e))?;

    run_migrations(&conn)?;
    Ok(Mutex::new(conn))
}

fn run_migrations(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS installed_skills (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            source TEXT NOT NULL DEFAULT 'local',
            source_url TEXT,
            version TEXT,
            license TEXT,
            author TEXT,
            skill_dir TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            installed_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT,
            format_score INTEGER,
            safety_level TEXT,
            quality_score INTEGER
        );

        CREATE TABLE IF NOT EXISTS agent_configs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            agent_type TEXT NOT NULL,
            base_path TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS install_mappings (
            skill_id TEXT NOT NULL,
            agent_id TEXT NOT NULL,
            installed_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (skill_id, agent_id),
            FOREIGN KEY (skill_id) REFERENCES installed_skills(id) ON DELETE CASCADE,
            FOREIGN KEY (agent_id) REFERENCES agent_configs(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS assessment_results (
            id TEXT PRIMARY KEY,
            skill_id TEXT NOT NULL,
            dimension TEXT NOT NULL,
            score INTEGER NOT NULL DEFAULT 0,
            issues TEXT NOT NULL DEFAULT '[]',
            assessed_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (skill_id) REFERENCES installed_skills(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS dedup_groups (
            id TEXT PRIMARY KEY,
            similarity REAL NOT NULL,
            level TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS dedup_group_skills (
            group_id TEXT NOT NULL,
            skill_id TEXT NOT NULL,
            PRIMARY KEY (group_id, skill_id),
            FOREIGN KEY (group_id) REFERENCES dedup_groups(id) ON DELETE CASCADE,
            FOREIGN KEY (skill_id) REFERENCES installed_skills(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS skill_conflicts (
            id TEXT PRIMARY KEY,
            skill_name TEXT NOT NULL,
            candidates TEXT NOT NULL,
            resolved INTEGER NOT NULL DEFAULT 0,
            kept_candidate_id TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );"
    ).map_err(|e| format!("Migration error: {}", e))?;
    Ok(())
}

// --- Installed Skills ---

fn map_skill_row(row: &rusqlite::Row) -> rusqlite::Result<InstalledSkill> {
    let agent_ids_str: Option<String> = row.get(15)?;
    let agent_ids = agent_ids_str
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    Ok(InstalledSkill {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        source: row.get(3)?,
        source_url: row.get(4)?,
        version: row.get(5)?,
        license: row.get(6)?,
        author: row.get(7)?,
        skill_dir: row.get(8)?,
        enabled: row.get::<_, i32>(9)? != 0,
        installed_at: row.get(10)?,
        updated_at: row.get(11)?,
        format_score: row.get(12)?,
        safety_level: row.get(13)?,
        quality_score: row.get(14)?,
        tags: Vec::new(),
        has_security_issues: false,
        agent_count: row.get(16)?,
        agent_ids,
    })
}

pub fn get_all_skills(conn: &DbConn) -> Result<Vec<InstalledSkill>, String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT s.id, s.name, s.description, s.source, s.source_url, s.version, s.license, s.author,
                s.skill_dir, s.enabled, s.installed_at, s.updated_at, s.format_score, s.safety_level, s.quality_score,
                GROUP_CONCAT(m.agent_id) as agent_ids,
                COUNT(m.agent_id) as agent_count
         FROM installed_skills s
         LEFT JOIN install_mappings m ON s.id = m.skill_id
         GROUP BY s.id
         ORDER BY s.name"
    ).map_err(|e| e.to_string())?;

    let skills = stmt.query_map([], map_skill_row)
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(skills)
}

pub fn get_skill_by_id(conn: &DbConn, id: &str) -> Result<Option<InstalledSkill>, String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT s.id, s.name, s.description, s.source, s.source_url, s.version, s.license, s.author,
                s.skill_dir, s.enabled, s.installed_at, s.updated_at, s.format_score, s.safety_level, s.quality_score,
                GROUP_CONCAT(m.agent_id) as agent_ids,
                COUNT(m.agent_id) as agent_count
         FROM installed_skills s
         LEFT JOIN install_mappings m ON s.id = m.skill_id
         WHERE s.id = ?1
         GROUP BY s.id"
    ).map_err(|e| e.to_string())?;

    let mut rows = stmt.query_map(params![id], map_skill_row)
        .map_err(|e| e.to_string())?;

    match rows.next() {
        Some(Ok(skill)) => Ok(Some(skill)),
        _ => Ok(None),
    }
}

pub fn insert_skill(conn: &DbConn, skill: &InstalledSkill) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO installed_skills (id, name, description, source, source_url, version, license, author, skill_dir, enabled, installed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(id) DO UPDATE SET
            name=excluded.name, description=excluded.description, source=excluded.source,
            source_url=excluded.source_url, version=excluded.version, license=excluded.license,
            author=excluded.author, skill_dir=excluded.skill_dir, updated_at=datetime('now')",
        params![
            skill.id, skill.name, skill.description, skill.source, skill.source_url,
            skill.version, skill.license, skill.author, skill.skill_dir,
            if skill.enabled { 1 } else { 0 }, skill.installed_at,
        ],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_skill(conn: &DbConn, id: &str) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM installed_skills WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn toggle_skill(conn: &DbConn, id: &str, enabled: bool) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute(
        "UPDATE installed_skills SET enabled = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![if enabled { 1 } else { 0 }, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// --- Agent Configs ---

pub fn get_all_agents(conn: &DbConn) -> Result<Vec<AgentConfig>, String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT id, name, agent_type, base_path, created_at FROM agent_configs ORDER BY name"
    ).map_err(|e| e.to_string())?;

    let agents = stmt.query_map([], |row| {
        Ok(AgentConfig {
            id: row.get(0)?,
            name: row.get(1)?,
            agent_type: row.get(2)?,
            base_path: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: None,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(agents)
}

pub fn insert_agent(conn: &DbConn, agent: &AgentConfig) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT INTO agent_configs (id, name, agent_type, base_path, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![agent.id, agent.name, agent.agent_type, agent.base_path, agent.created_at],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn update_agent(conn: &DbConn, agent: &AgentConfig) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute(
        "UPDATE agent_configs SET name=?1, agent_type=?2, base_path=?3 WHERE id=?4",
        params![agent.name, agent.agent_type, agent.base_path, agent.id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_agent(conn: &DbConn, id: &str) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM agent_configs WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

// --- Install Mappings ---

pub fn get_mappings_for_skill(conn: &DbConn, skill_id: &str) -> Result<Vec<String>, String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT agent_id FROM install_mappings WHERE skill_id = ?1"
    ).map_err(|e| e.to_string())?;

    let ids = stmt.query_map(params![skill_id], |row| {
        row.get::<_, String>(0)
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(ids)
}

pub fn add_mapping(conn: &DbConn, skill_id: &str, agent_id: &str) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR IGNORE INTO install_mappings (skill_id, agent_id) VALUES (?1, ?2)",
        params![skill_id, agent_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove_mappings_for_skill(conn: &DbConn, skill_id: &str) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM install_mappings WHERE skill_id = ?1", params![skill_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn remove_mapping(conn: &DbConn, skill_id: &str, agent_id: &str) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute(
        "DELETE FROM install_mappings WHERE skill_id = ?1 AND agent_id = ?2",
        params![skill_id, agent_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// --- Settings ---

pub fn get_setting(conn: &DbConn, key: &str) -> Result<Option<String>, String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare("SELECT value FROM app_settings WHERE key = ?1")
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query_map(params![key], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    match rows.next() {
        Some(Ok(val)) => Ok(Some(val)),
        _ => Ok(None),
    }
}

pub fn set_setting(conn: &DbConn, key: &str, value: &str) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// --- Assessment ---

pub fn insert_assessment(conn: &DbConn, result: &AssessmentResult) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR REPLACE INTO assessment_results (id, skill_id, dimension, score, issues, assessed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![result.id, result.skill_id, result.dimension, result.score, result.issues, result.assessed_at],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_assessments_for_skill(conn: &DbConn, skill_id: &str) -> Result<Vec<AssessmentResult>, String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT id, skill_id, dimension, score, issues, assessed_at FROM assessment_results WHERE skill_id = ?1"
    ).map_err(|e| e.to_string())?;

    let results = stmt.query_map(params![skill_id], |row| {
        Ok(AssessmentResult {
            id: row.get(0)?,
            skill_id: row.get(1)?,
            dimension: row.get(2)?,
            score: row.get(3)?,
            issues: row.get(4)?,
            assessed_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    Ok(results)
}

// --- Dedup ---

pub fn insert_dedup_group(conn: &DbConn, id: &str, similarity: f64, level: &str) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR REPLACE INTO dedup_groups (id, similarity, level) VALUES (?1, ?2, ?3)",
        params![id, similarity, level],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn add_skill_to_dedup_group(conn: &DbConn, group_id: &str, skill_id: &str) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR IGNORE INTO dedup_group_skills (group_id, skill_id) VALUES (?1, ?2)",
        params![group_id, skill_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

// --- Skill Conflicts ---

pub fn get_unresolved_conflicts(conn: &DbConn) -> Result<Vec<SkillConflict>, String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT id, skill_name, candidates, resolved, kept_candidate_id, created_at
         FROM skill_conflicts WHERE resolved = 0 ORDER BY created_at"
    ).map_err(|e| e.to_string())?;

    let conflicts = stmt.query_map([], |row| {
        let candidates_str: String = row.get(2)?;
        let candidates: Vec<ConflictCandidate> = serde_json::from_str(&candidates_str)
            .unwrap_or_default();
        Ok(SkillConflict {
            id: row.get(0)?,
            skill_name: row.get(1)?,
            candidates,
            resolved: row.get::<_, i32>(3)? != 0,
            kept_candidate_id: row.get(4)?,
            created_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    Ok(conflicts)
}

pub fn insert_conflict(conn: &DbConn, conflict: &SkillConflict) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    let candidates_json = serde_json::to_string(&conflict.candidates)
        .map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR REPLACE INTO skill_conflicts (id, skill_name, candidates, resolved, kept_candidate_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            conflict.id,
            conflict.skill_name,
            candidates_json,
            if conflict.resolved { 1 } else { 0 },
            conflict.kept_candidate_id,
            conflict.created_at,
        ],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn resolve_conflict(
    conn: &DbConn,
    conflict_id: &str,
    kept_candidate_id: &str,
) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute(
        "UPDATE skill_conflicts SET resolved = 1, kept_candidate_id = ?1 WHERE id = ?2",
        params![kept_candidate_id, conflict_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_conflicts_for_skill_name(conn: &DbConn, skill_name: &str) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute(
        "DELETE FROM skill_conflicts WHERE skill_name = ?1",
        params![skill_name],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn clear_resolved_conflicts(conn: &DbConn) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM skill_conflicts WHERE resolved = 1", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn clear_dedup_groups(conn: &DbConn) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    db.execute_batch("DELETE FROM dedup_group_skills; DELETE FROM dedup_groups;")
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_dedup_groups_with_skills(conn: &DbConn) -> Result<Vec<(String, f64, String, Vec<InstalledSkill>)>, String> {
    let db = conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare(
        "SELECT g.id, g.similarity, g.level, dgs.skill_id
         FROM dedup_groups g
         JOIN dedup_group_skills dgs ON g.id = dgs.group_id
         ORDER BY g.similarity DESC"
    ).map_err(|e| e.to_string())?;

    let rows: Vec<(String, f64, String, String)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();

    drop(stmt);

    let mut groups: Vec<(String, f64, String, Vec<InstalledSkill>)> = Vec::new();
    let mut current_group_id = String::new();
    let mut current_group: Option<(String, f64, String, Vec<InstalledSkill>)> = None;

    for (gid, sim, lvl, sid) in rows {
        if gid != current_group_id {
            if let Some(g) = current_group.take() {
                groups.push(g);
            }
            current_group_id = gid.clone();
            current_group = Some((gid.clone(), sim, lvl.clone(), Vec::new()));
        }

        if let Some(ref mut g) = current_group {
            if let Ok(Some(skill)) = get_skill_by_id_internal(&db, &sid) {
                g.3.push(skill);
            }
        }
    }
    if let Some(g) = current_group.take() {
        groups.push(g);
    }

    Ok(groups)
}

fn get_skill_by_id_internal(db: &Connection, id: &str) -> Result<Option<InstalledSkill>, String> {
    let mut stmt = db.prepare(
        "SELECT s.id, s.name, s.description, s.source, s.source_url, s.version, s.license, s.author,
                s.skill_dir, s.enabled, s.installed_at, s.updated_at, s.format_score, s.safety_level, s.quality_score,
                GROUP_CONCAT(m.agent_id) as agent_ids,
                COUNT(m.agent_id) as agent_count
         FROM installed_skills s
         LEFT JOIN install_mappings m ON s.id = m.skill_id
         WHERE s.id = ?1
         GROUP BY s.id"
    ).map_err(|e| e.to_string())?;

    let mut rows = stmt.query_map(params![id], |row| {
        let agent_ids_str: Option<String> = row.get(15)?;
        let agent_ids = agent_ids_str
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        Ok(InstalledSkill {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            source: row.get(3)?,
            source_url: row.get(4)?,
            version: row.get(5)?,
            license: row.get(6)?,
            author: row.get(7)?,
            skill_dir: row.get(8)?,
            enabled: row.get::<_, i32>(9)? != 0,
            installed_at: row.get(10)?,
            updated_at: row.get(11)?,
            format_score: row.get(12)?,
            safety_level: row.get(13)?,
            quality_score: row.get(14)?,
            tags: Vec::new(),
            has_security_issues: false,
            agent_count: row.get(16)?,
            agent_ids,
        })
    }).map_err(|e| e.to_string())?;

    match rows.next() {
        Some(Ok(skill)) => Ok(Some(skill)),
        _ => Ok(None),
    }
}

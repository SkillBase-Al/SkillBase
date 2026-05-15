use std::fs;
use tauri::State;
use uuid::Uuid;
use crate::db::models::AssessmentResult;
use crate::db::repository::{self, DbConn};

#[tauri::command]
pub fn assess_format(conn: State<DbConn>, skill_id: String) -> Result<crate::checker::format_checker::FormatResult, String> {
    let skill = repository::get_skill_by_id(&conn, &skill_id)?
        .ok_or_else(|| "Skill not found".to_string())?;

    let skill_md_path = std::path::Path::new(&skill.skill_dir).join("SKILL.md");
    let content = fs::read_to_string(&skill_md_path)
        .map_err(|e| format!("Failed to read SKILL.md: {}", e))?;

    let result = crate::checker::format_checker::assess(&content);

    // Save result
    let assessment = AssessmentResult {
        id: Uuid::new_v4().to_string(),
        skill_id: skill_id.clone(),
        dimension: "format".into(),
        score: result.score,
        issues: serde_json::to_string(&result.issues).unwrap_or_default(),
        assessed_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    repository::insert_assessment(&conn, &assessment)?;

    // Update format_score on skill
    if let Ok(db) = conn.lock() {
        let _ = db.execute(
            "UPDATE installed_skills SET format_score = ?1 WHERE id = ?2",
            rusqlite::params![result.score, skill_id],
        );
    }

    Ok(result)
}

#[tauri::command]
pub fn assess_security(conn: State<DbConn>, skill_id: String) -> Result<crate::checker::security_checker::SecurityResult, String> {
    let skill = repository::get_skill_by_id(&conn, &skill_id)?
        .ok_or_else(|| "Skill not found".to_string())?;

    let skill_dir = std::path::Path::new(&skill.skill_dir);
    let mut content = String::new();

    // Read SKILL.md for security scanning
    let md_path = skill_dir.join("SKILL.md");
    if md_path.exists() {
        content.push_str(&fs::read_to_string(&md_path).unwrap_or_default());
    }

    // Also scan scripts/ directory
    let scripts_dir = skill_dir.join("scripts");
    if scripts_dir.exists() {
        if let Ok(entries) = fs::read_dir(&scripts_dir) {
            for entry in entries.flatten() {
                if let Ok(c) = fs::read_to_string(entry.path()) {
                    content.push('\n');
                    content.push_str(&c);
                }
            }
        }
    }

    let result = crate::checker::security_checker::assess(&content);

    // Save result
    let assessment = AssessmentResult {
        id: Uuid::new_v4().to_string(),
        skill_id: skill_id.clone(),
        dimension: "security".into(),
        score: result.score,
        issues: serde_json::to_string(&result.issues).unwrap_or_default(),
        assessed_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    repository::insert_assessment(&conn, &assessment)?;

    // Update safety_level on skill
    if let Ok(db) = conn.lock() {
        let _ = db.execute(
            "UPDATE installed_skills SET safety_level = ?1 WHERE id = ?2",
            rusqlite::params![result.level, skill_id],
        );
    }

    Ok(result)
}

#[tauri::command]
pub fn batch_assess(conn: State<DbConn>) -> Result<crate::db::models::AssessmentSummary, String> {
    let skills = repository::get_all_skills(&conn)?;
    let mut total = 0;
    let mut passed = 0;
    let mut total_format = 0i64;
    let total_quality = 0i64;
    let mut safe_count = 0;
    let mut warning_count = 0;
    let mut dangerous_count = 0;

    for skill in &skills {
        let skill_dir = std::path::Path::new(&skill.skill_dir);
        let md_path = skill_dir.join("SKILL.md");

        if md_path.exists() {
            if let Ok(content) = fs::read_to_string(&md_path) {
                let fmt_result = crate::checker::format_checker::assess(&content);
                total_format += fmt_result.score as i64;

                let assessment = AssessmentResult {
                    id: Uuid::new_v4().to_string(),
                    skill_id: skill.id.clone(),
                    dimension: "format".into(),
                    score: fmt_result.score,
                    issues: serde_json::to_string(&fmt_result.issues).unwrap_or_default(),
                    assessed_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                };
                let _ = repository::insert_assessment(&conn, &assessment);

                // Update format_score on installed_skills
                if let Ok(db) = conn.lock() {
                    let _ = db.execute(
                        "UPDATE installed_skills SET format_score = ?1 WHERE id = ?2",
                        rusqlite::params![fmt_result.score, skill.id],
                    );
                }

                if fmt_result.score >= 60 { passed += 1; }
            }
        }

        // Security scan
        let mut sec_content = String::new();
        if let Ok(c) = fs::read_to_string(&md_path) { sec_content.push_str(&c); }
        let scripts_dir = skill_dir.join("scripts");
        if scripts_dir.exists() {
            if let Ok(entries) = fs::read_dir(&scripts_dir) {
                for entry in entries.flatten() {
                    if let Ok(c) = fs::read_to_string(entry.path()) {
                        sec_content.push('\n');
                        sec_content.push_str(&c);
                    }
                }
            }
        }

        let sec_result = crate::checker::security_checker::assess(&sec_content);
        match sec_result.level.as_str() {
            "Safe" => safe_count += 1,
            "Warning" => warning_count += 1,
            _ => dangerous_count += 1,
        }

        // Save security assessment result
        let sec_assessment = AssessmentResult {
            id: Uuid::new_v4().to_string(),
            skill_id: skill.id.clone(),
            dimension: "security".into(),
            score: sec_result.score,
            issues: serde_json::to_string(&sec_result.issues).unwrap_or_default(),
            assessed_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        let _ = repository::insert_assessment(&conn, &sec_assessment);

        // Update safety_level on installed_skills
        if let Ok(db) = conn.lock() {
            let _ = db.execute(
                "UPDATE installed_skills SET safety_level = ?1 WHERE id = ?2",
                rusqlite::params![sec_result.level, skill.id],
            );
        }


        total += 1;
    }

    Ok(crate::db::models::AssessmentSummary {
        total,
        passed,
        average_format_score: if total > 0 { total_format as f64 / total as f64 } else { 0.0 },
        average_quality_score: if total > 0 { total_quality as f64 / total as f64 } else { 0.0 },
        safe_count,
        warning_count,
        dangerous_count,
    })
}

#[tauri::command]
pub fn get_assessment_results(conn: State<DbConn>, skill_id: String) -> Result<Vec<AssessmentResult>, String> {
    repository::get_assessments_for_skill(&conn, &skill_id)
}

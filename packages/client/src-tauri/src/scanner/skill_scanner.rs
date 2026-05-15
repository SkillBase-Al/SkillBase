use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedSkill {
    pub path: PathBuf,
    pub name: String,
    pub content_hash: String,
    pub modified_at: u64,
}

/// Scan a directory for SKILL.md files using gitignore-style patterns
pub fn scan_directory(dir: &Path) -> Result<Vec<ScannedSkill>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut skills = Vec::new();

    // Use ignore crate to respect .gitignore patterns
    let walker = WalkBuilder::new(dir)
        .standard_filters(true)
        .build();

    for entry in walker {
        let entry = entry.map_err(|e| format!("Walk error: {}", e))?;
        let path = entry.path();

        if path.file_name().and_then(|n| n.to_str()) == Some("SKILL.md") {
            let content = fs::read_to_string(path)
                .map_err(|e| format!("Read error {}: {}", path.display(), e))?;

            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            let hash = format!("{:x}", hasher.finalize());

            let metadata = fs::metadata(path)
                .map_err(|e| format!("Metadata error {}: {}", path.display(), e))?;

            let modified = metadata.modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            // Determine skill name from parent directory name
            let name = path.parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            skills.push(ScannedSkill {
                path: path.to_path_buf(),
                name,
                content_hash: hash,
                modified_at: modified,
            });
        }
    }

    Ok(skills)
}

/// Scan the default skills directory
pub fn scan_default() -> Result<Vec<ScannedSkill>, String> {
    let dir = crate::utils::paths::skills_dir();
    scan_directory(&dir)
}

/// Detect new, modified, and deleted skills since last scan
pub fn detect_changes(
    current: &[ScannedSkill],
    previous_hashes: &HashMap<String, String>,
) -> (Vec<ScannedSkill>, Vec<ScannedSkill>, Vec<String>) {
    let mut new_skills = Vec::new();
    let mut modified_skills = Vec::new();
    let current_map: HashMap<String, &ScannedSkill> = current.iter()
        .map(|s| (s.name.clone(), s))
        .collect();

    for skill in current {
        match previous_hashes.get(&skill.name) {
            None => new_skills.push(skill.clone()),
            Some(old_hash) if old_hash != &skill.content_hash => modified_skills.push(skill.clone()),
            _ => {}
        }
    }

    let deleted: Vec<String> = previous_hashes.keys()
        .filter(|name| !current_map.contains_key(*name))
        .cloned()
        .collect();

    (new_skills, modified_skills, deleted)
}

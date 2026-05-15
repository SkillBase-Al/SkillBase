use std::path::{Path, PathBuf};
use std::fs;

/// Get the SkillBase base directory (~/.skillbase)
pub fn base_dir() -> PathBuf {
    let home = dirs_next().unwrap_or_else(|| PathBuf::from("."));
    home.join(".skillbase")
}

fn dirs_next() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
        .or_else(|| std::env::var("USERPROFILE").ok().map(PathBuf::from))
}

/// Full path to the skills storage directory
pub fn skills_dir() -> PathBuf {
    base_dir().join("skills")
}

/// Full path to the skillsets directory
pub fn skillsets_dir() -> PathBuf {
    base_dir().join("skillsets")
}

/// Full path to the SQLite index database
pub fn index_db_path() -> PathBuf {
    base_dir().join("index.db")
}

/// Full path to the agents config JSON
pub fn agents_json_path() -> PathBuf {
    base_dir().join("agents.json")
}

/// Full path to the logs directory
pub fn logs_dir() -> PathBuf {
    base_dir().join("logs")
}

/// Ensure all base directories exist, creating them if necessary
pub fn ensure_base_dirs(app_dir: &Path) -> std::io::Result<()> {
    let dirs = [
        base_dir(),
        skills_dir(),
        skillsets_dir(),
        logs_dir(),
    ];
    for d in &dirs {
        fs::create_dir_all(d)?;
    }
    Ok(())
}

/// Split a ';'-separated base_path string into individual expanded paths.
/// Each segment is trimmed and `~` is expanded to `$HOME`.
pub fn split_base_paths(base_path: &str) -> Vec<PathBuf> {
    base_path
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.starts_with("~/") {
                if let Some(home) = std::env::var("HOME").ok() {
                    PathBuf::from(home).join(&s[2..])
                } else {
                    PathBuf::from(s)
                }
            } else {
                PathBuf::from(s)
            }
        })
        .collect()
}

/// Get agent-specific skill directory.
/// base_path is now the full skill directory path (e.g. ~/.cursor/skills/).
pub fn get_agent_skill_dir(_agent_type: &str, base_path: &Path) -> PathBuf {
    base_path.to_path_buf()
}

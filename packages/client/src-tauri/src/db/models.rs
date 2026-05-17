use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: String,
    pub source_url: Option<String>,
    pub version: Option<String>,
    pub license: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "path")]
    pub skill_dir: String,
    pub enabled: bool,
    pub installed_at: String,
    pub updated_at: Option<String>,
    pub format_score: Option<i32>,
    pub safety_level: Option<String>,
    pub quality_score: Option<i32>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub has_security_issues: bool,
    #[serde(default)]
    pub agent_count: i32,
    #[serde(default)]
    pub agent_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub agent_type: String,
    pub base_path: String,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallMapping {
    pub skill_id: String,
    pub agent_id: String,
    pub installed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub server_url: String,
    pub proxy_url: String,
    #[serde(default)]
    pub auto_scan: bool,
    #[serde(default)]
    pub auto_assess: bool,
    #[serde(default)]
    pub scan_paths: Vec<String>,
    #[serde(default)]
    pub sidebar_collapsed: bool,
    #[serde(default)]
    pub first_run_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResult {
    pub id: String,
    pub skill_id: String,
    pub dimension: String,
    pub score: i32,
    pub issues: String,
    pub assessed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub source: Option<String>,
    pub source_url: Option<String>,
    pub tags: Vec<String>,
    pub requires: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupGroup {
    pub id: String,
    pub similarity: f64,
    pub level: String,
    pub skills: Vec<InstalledSkill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentSummary {
    pub total: i32,
    pub passed: i32,
    pub average_format_score: f64,
    pub average_quality_score: f64,
    pub safe_count: i32,
    pub warning_count: i32,
    pub dangerous_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictCandidate {
    pub id: String,
    pub skill_id: Option<String>,
    pub skill_dir: String,
    pub agent_id: String,
    pub agent_name: String,
    pub agent_type: String,
    pub version: Option<String>,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feedback {
    pub id: String,
    pub title: String,
    pub description: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillConflict {
    pub id: String,
    pub skill_name: String,
    pub candidates: Vec<ConflictCandidate>,
    pub resolved: bool,
    pub kept_candidate_id: Option<String>,
    pub created_at: String,
}

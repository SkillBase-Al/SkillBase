use serde::{Deserialize, Serialize};
use serde_yaml;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub severity: String,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatResult {
    pub score: i32,
    pub issues: Vec<Issue>,
    pub metadata: Option<serde_yaml::Value>,
}

/// Parse YAML frontmatter from SKILL.md content
fn parse_frontmatter(content: &str) -> Option<(serde_yaml::Value, usize)> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }

    let end = content[3..].find("\n---")?;
    let yaml_str = &content[3..3 + end];
    let yaml_val: serde_yaml::Value = serde_yaml::from_str(yaml_str).ok()?;
    Some((yaml_val, 3 + end + 5)) // +5 for \n---\n
}

/// Validate SKILL.md format compliance scoring 0-100
pub fn assess(content: &str) -> FormatResult {
    let mut issues = Vec::new();
    let mut score = 100i32;

    let (yaml, _) = match parse_frontmatter(content) {
        Some(v) => v,
        None => {
            issues.push(Issue {
                severity: "error".into(),
                field: "frontmatter".into(),
                message: "Missing or invalid YAML frontmatter (must start and end with ---)".into(),
            });
            return FormatResult { score: 0, issues, metadata: None };
        }
    };

    let map = match yaml.as_mapping() {
        Some(m) => m,
        None => {
            issues.push(Issue {
                severity: "error".into(),
                field: "frontmatter".into(),
                message: "Frontmatter must be a YAML mapping".into(),
            });
            return FormatResult { score: 0, issues, metadata: None };
        }
    };

    // Check name field
    let name = map.get(&serde_yaml::Value::String("name".into()));
    match name {
        Some(serde_yaml::Value::String(n)) => {
            if n.is_empty() {
                score -= 20;
                issues.push(Issue {
                    severity: "error".into(),
                    field: "name".into(),
                    message: "Field 'name' must not be empty".into(),
                });
            } else if n.chars().any(|c| c.is_uppercase()) || n.contains('_') {
                score -= 10;
                issues.push(Issue {
                    severity: "warning".into(),
                    field: "name".into(),
                    message: "Field 'name' should be lowercase with hyphens (kebab-case)".into(),
                });
            }
        }
        None => {
            score -= 20;
            issues.push(Issue {
                severity: "error".into(),
                field: "name".into(),
                message: "Required field 'name' is missing".into(),
            });
        }
        _ => {
            score -= 10;
            issues.push(Issue {
                severity: "warning".into(),
                field: "name".into(),
                message: "Field 'name' must be a string".into(),
            });
        }
    }

    // Check description field
    let desc = map.get(&serde_yaml::Value::String("description".into()));
    match desc {
        Some(serde_yaml::Value::String(d)) => {
            if d.len() < 50 {
                score -= 15;
                issues.push(Issue {
                    severity: "warning".into(),
                    field: "description".into(),
                    message: format!("Description too short ({} chars, minimum 50)", d.len()),
                });
            }
            if d.len() > 500 {
                score -= 5;
                issues.push(Issue {
                    severity: "info".into(),
                    field: "description".into(),
                    message: format!("Description is very long ({} chars)", d.len()),
                });
            }
        }
        None => {
            score -= 20;
            issues.push(Issue {
                severity: "error".into(),
                field: "description".into(),
                message: "Required field 'description' is missing".into(),
            });
        }
        _ => {}
    }

    // Check version field
    let version = map.get(&serde_yaml::Value::String("version".into()));
    match version {
        Some(serde_yaml::Value::String(v)) => {
            if !v.starts_with(|c: char| c.is_ascii_digit()) || v.matches('.').count() < 2 {
                if v != "latest" {
                    score -= 10;
                    issues.push(Issue {
                        severity: "warning".into(),
                        field: "version".into(),
                        message: format!("Version '{}' does not follow semver (e.g., 1.0.0)", v),
                    });
                }
            }
        }
        None => {
            score -= 5;
            issues.push(Issue {
                severity: "info".into(),
                field: "version".into(),
                message: "Recommended field 'version' is missing".into(),
            });
        }
        _ => {}
    }

    // Check tags field
    let tags = map.get(&serde_yaml::Value::String("tags".into()));
    match tags {
        Some(serde_yaml::Value::Sequence(t)) if t.is_empty() => {
            score -= 5;
            issues.push(Issue {
                severity: "info".into(),
                field: "tags".into(),
                message: "Field 'tags' is empty; adding tags improves discoverability".into(),
            });
        }
        None => {
            score -= 5;
            issues.push(Issue {
                severity: "info".into(),
                field: "tags".into(),
                message: "Recommended field 'tags' is missing".into(),
            });
        }
        _ => {}
    }

    // Clamp score
    score = score.clamp(0, 100);

    FormatResult { score, issues, metadata: Some(yaml) }
}

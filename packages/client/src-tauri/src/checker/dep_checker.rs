use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepIssue {
    pub severity: String,
    pub package: Option<String>,
    pub manager: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepResult {
    pub score: i32,
    pub has_dependencies: bool,
    pub issues: Vec<DepIssue>,
}

const DEP_PATTERNS: &[(&str, &str)] = &[
    ("pip install", "pip"),
    ("npm install", "npm"),
    ("brew install", "brew"),
    ("apt-get install", "apt"),
    ("cargo install", "cargo"),
    ("go install", "go"),
    ("gem install", "gem"),
    ("docker pull", "docker"),
    ("docker run", "docker"),
    ("snap install", "snap"),
];

/// Check for dependency declarations in SKILL.md and scripts
pub fn assess(content: &str) -> DepResult {
    let mut issues = Vec::new();
    let mut has_dependencies = false;

    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        for (pattern, manager) in DEP_PATTERNS {
            if let Some(pos) = line.find(pattern) {
                has_dependencies = true;
                // Extract the package dependency (everything after the command)
                let after = &line[pos + pattern.len()..].trim();
                if after.is_empty() || after.starts_with(&['-', '#', '/'][..]) {
                    issues.push(DepIssue {
                        severity: "warning".into(),
                        package: None,
                        manager: Some(manager.to_string()),
                        message: format!("Found {} command but no package specified on line {}", manager, i + 1),
                    });
                } else {
                    // Extract package name (first word)
                    let pkg = after.split_whitespace().next().unwrap_or("").trim();
                    if !pkg.is_empty() && !pkg.starts_with('-') {
                        issues.push(DepIssue {
                            severity: "info".into(),
                            package: Some(pkg.to_string()),
                            manager: Some(manager.to_string()),
                            message: format!("Requires {} package '{}'", manager, pkg),
                        });
                    }
                }
            }
        }

        // Check for requires/depends_on YAML-like metadata
        if line.trim().starts_with("requires:") || line.trim().starts_with("dependencies:") {
            has_dependencies = true;
            let rest = line.split(':').nth(1).unwrap_or("").trim();
            if rest.is_empty() {
                issues.push(DepIssue {
                    severity: "info".into(),
                    package: None,
                    manager: None,
                    message: format!("Line {}: dependency field declared but empty", i + 1),
                });
            }
        }
    }

    let score = if has_dependencies {
        let warnings = issues.iter().filter(|i| i.severity == "warning").count();
        (100 - warnings as i32 * 20).clamp(0, 100)
    } else {
        0 // N/A - no deps declared
    };

    DepResult { score, has_dependencies, issues }
}

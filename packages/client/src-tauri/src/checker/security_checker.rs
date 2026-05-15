use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub line: usize,
    pub content: String,
    pub risk: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityResult {
    pub level: String, // Safe / Warning / Dangerous
    pub score: i32,
    pub issues: Vec<SecurityIssue>,
}

// Known dangerous patterns
const DANGEROUS_PATTERNS: &[(&str, &str)] = &[
    (r"rm\s+-rf\s+/", "Recursive root filesystem deletion"),
    (r"rm\s+-rf\s+~", "Recursive home directory deletion"),
    (r"mkfs\.", "Filesystem format command"),
    (r"dd\s+if=.*of=/dev/", "Direct block device write"),
    (r":\(\)\s*\{[^}]*\};\s*:", "Fork bomb detected"),
    (r"chmod\s+-R\s+777\s+/", "Recursive world-writable permission on root"),
    (r"base64.*\|\s*(bash|sh|zsh)", "Base64 encoded command execution"),
    (r"eval\s+\$\(.*curl\|.*bash", "Eval with curl pipe to shell"),
    (r"\|\s*(bash|sh|zsh)\s*$", "Pipe to shell"),
    (r"wget.*-O[^=].*\|\s*(bash|sh)", "Wget pipe to shell"),
    (r"curl.*\|\s*(bash|sh|zsh)", "Curl pipe to shell"),
];

const WARNING_PATTERNS: &[(&str, &str)] = &[
    (r"sudo\s+", "Uses sudo - may require elevated privileges"),
    (r"chmod\s+777", "Sets world-writable permissions"),
    (r"chown\s+-R", "Recursive owner change"),
    (r">\s*/etc/", "Writing to system configuration"),
    (r"apt-get\s+(install|remove|purge)", "System package modification"),
    (r"brew\s+(install|remove|uninstall)", "Homebrew package modification"),
    (r"npm\s+(install|publish)\s*-g", "Global npm package installation"),
    (r"pip\s+install", "Python package installation"),
    (r"export\s+\w+=\$\(.*\)", "Dynamic environment variable export"),
];

/// Scan scripts content for dangerous patterns
pub fn assess(content: &str) -> SecurityResult {
    let mut issues = Vec::new();
    let mut dangerous_count = 0;

    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;

        for (pattern, desc) in DANGEROUS_PATTERNS {
            if line.contains(&pattern[1..pattern.len()-1]) || regex_match(line, pattern) {
                issues.push(SecurityIssue {
                    line: line_num,
                    content: line.trim().to_string(),
                    risk: "dangerous".into(),
                    description: desc.to_string(),
                });
                dangerous_count += 1;
                break;
            }
        }

        if dangerous_count > 20 {
            break; // Stop scanning if already very dangerous
        }
    }

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let already_found = issues.iter().any(|iss| iss.line == line_num);
        if already_found { continue; }

        for (pattern, desc) in WARNING_PATTERNS {
            if line.contains(&pattern[1..pattern.len()-1]) || regex_match(line, pattern) {
                issues.push(SecurityIssue {
                    line: line_num,
                    content: line.trim().to_string(),
                    risk: "warning".into(),
                    description: desc.to_string(),
                });
                break;
            }
        }
    }

    // Determine overall level
    let level = if dangerous_count > 0 {
        "Dangerous"
    } else if issues.iter().any(|i| i.risk == "warning") {
        "Warning"
    } else {
        "Safe"
    };

    // Score: 100 base, -20 per dangerous, -5 per warning
    let warning_count = issues.iter().filter(|i| i.risk == "warning").count() as i32;
    let score = (100 - dangerous_count * 20 - warning_count * 5).clamp(0, 100);

    SecurityResult { level: level.to_string(), score, issues }
}

fn regex_match(line: &str, pattern: &str) -> bool {
    // Simple pattern matching without regex crate dependency
    // Matches patterns like "rm -rf /" but not "rm -rf /something/safe"
    let stripped = pattern.trim_start_matches('r').trim_end_matches('/');
    line.contains(stripped.trim_start_matches(|c: char| c == '(' || c == '?' || c == ':' || c == '\\' || c == '.'))
}

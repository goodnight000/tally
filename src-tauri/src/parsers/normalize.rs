/// Normalize a Claude Code encoded project path (e.g. "-Users-charleszheng-Desktop-project")
/// back to an absolute path (e.g. "/Users/charleszheng/Desktop/project")
pub fn decode_claude_project_path(encoded: &str) -> String {
    if encoded.starts_with('-') {
        // Replace leading dash with /, then replace remaining dashes with /
        // But be careful: directory names with dashes exist, so we use a heuristic
        // Claude Code encodes path separators as single dashes
        format!("/{}", &encoded[1..]).replace('-', "/")
    } else {
        encoded.to_string()
    }
}

/// Derive a project name from a path and optional git origin URL
pub fn derive_project_name(project_path: Option<&str>, git_origin_url: Option<&str>) -> Option<String> {
    // Prefer git repo name if available
    if let Some(url) = git_origin_url {
        if !url.is_empty() {
            // Extract repo name from URLs like:
            // https://github.com/user/repo.git
            // git@github.com:user/repo.git
            let name = url
                .trim_end_matches('/')
                .trim_end_matches(".git")
                .rsplit('/')
                .next()
                .or_else(|| url.rsplit(':').next())
                .unwrap_or(url);
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    // Fall back to last path component
    if let Some(path) = project_path {
        if !path.is_empty() {
            let name = path
                .trim_end_matches('/')
                .rsplit('/')
                .next()
                .unwrap_or(path);
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    None
}

/// Classify a Codex source field value
/// Returns (source_type, parent_thread_id)
pub fn classify_codex_source(source: &str) -> (String, Option<String>) {
    match source {
        "cli" => ("cli".to_string(), None),
        "vscode" => ("vscode".to_string(), None),
        s if s.starts_with('{') => {
            // Try to parse as JSON for subagent detection
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(s) {
                if let Some(subagent) = val.get("subagent") {
                    let parent_id = subagent
                        .get("thread_spawn")
                        .and_then(|ts| ts.get("parent_thread_id"))
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_string());
                    return ("subagent".to_string(), parent_id);
                }
            }
            ("unknown".to_string(), None)
        }
        _ => (source.to_string(), None),
    }
}

/// Convert Unix timestamp (seconds) to ISO 8601 UTC string
pub fn unix_to_iso8601(timestamp: i64) -> String {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

/// Convert Unix timestamp (milliseconds) to ISO 8601 UTC string
pub fn unix_ms_to_iso8601(timestamp_ms: i64) -> String {
    unix_to_iso8601(timestamp_ms / 1000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_claude_project_path() {
        assert_eq!(
            decode_claude_project_path("-Users-charleszheng-Desktop-exam2"),
            "/Users/charleszheng/Desktop/exam2"
        );
    }

    #[test]
    fn test_derive_project_name_from_git_url() {
        assert_eq!(
            derive_project_name(Some("/home/user/myrepo"), Some("https://github.com/user/myrepo.git")),
            Some("myrepo".to_string())
        );
    }

    #[test]
    fn test_derive_project_name_from_path() {
        assert_eq!(
            derive_project_name(Some("/Users/charles/Desktop/Ideas/Tally"), None),
            Some("Tally".to_string())
        );
    }

    #[test]
    fn test_classify_codex_source_cli() {
        let (src, parent) = classify_codex_source("cli");
        assert_eq!(src, "cli");
        assert!(parent.is_none());
    }

    #[test]
    fn test_classify_codex_source_subagent() {
        let json = r#"{"subagent":{"thread_spawn":{"parent_thread_id":"abc-123","depth":1}}}"#;
        let (src, parent) = classify_codex_source(json);
        assert_eq!(src, "subagent");
        assert_eq!(parent, Some("abc-123".to_string()));
    }

    #[test]
    fn test_unix_to_iso8601() {
        assert_eq!(unix_to_iso8601(1770164494), "2026-02-04T00:21:34Z");
    }
}

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::db::models::{Request, Session};
use crate::parsers::normalize::derive_project_name;

/// A parsed Claude Code session with its requests
pub struct ParsedClaudeSession {
    pub session: Session,
    pub requests: Vec<Request>,
}

/// Parse a single Claude Code JSONL file
/// Only extracts assistant messages with usage data (data minimization)
pub fn parse_claude_jsonl(
    file_path: &Path,
    byte_offset: u64,
) -> Result<(Vec<ParsedClaudeSession>, u64), String> {
    let mut file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open {}: {}", file_path.display(), e))?;

    let file_size = file.metadata()
        .map_err(|e| format!("Failed to get file metadata: {}", e))?
        .len();

    if byte_offset >= file_size {
        return Ok((Vec::new(), file_size));
    }

    if byte_offset > 0 {
        file.seek(SeekFrom::Start(byte_offset))
            .map_err(|e| format!("Failed to seek: {}", e))?;
    }

    let reader = BufReader::new(file);

    // Group by session ID
    let mut sessions_map: HashMap<String, ParsedClaudeSession> = HashMap::new();

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break, // Stop on incomplete line at EOF
        };

        if line.trim().is_empty() {
            continue;
        }

        let entry: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Only process assistant messages with usage data
        let entry_type = entry.get("type").and_then(|t| t.as_str());
        if entry_type != Some("assistant") {
            continue;
        }

        let message = match entry.get("message") {
            Some(m) => m,
            None => continue,
        };

        let usage = match message.get("usage") {
            Some(u) => u,
            None => continue,
        };

        // Extract metadata
        let session_id = entry.get("sessionId").and_then(|v| v.as_str()).unwrap_or("unknown");
        let timestamp = entry.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
        let model = message.get("model").and_then(|v| v.as_str());
        let cwd = entry.get("cwd").and_then(|v| v.as_str());
        let version = entry.get("version").and_then(|v| v.as_str());
        let git_branch = entry.get("gitBranch").and_then(|v| v.as_str());
        let uuid = entry.get("uuid").and_then(|v| v.as_str()).unwrap_or("");
        let is_sidechain = entry.get("isSidechain").and_then(|v| v.as_bool()).unwrap_or(false);
        let agent_id = entry.get("agentId").and_then(|v| v.as_str());

        // Extract token counts from Claude API
        // Claude reports: input_tokens (non-cached only), cache_read and cache_creation as separate additive fields
        // We normalize: input_tokens = total input sent to model (input + cache_read + cache_creation)
        let raw_input = usage.get("input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
        let output_tokens = usage.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
        let cache_read = usage.get("cache_read_input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
        let cache_creation = usage.get("cache_creation_input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
        // Normalized: total input = raw input + all cache tokens
        let normalized_input = raw_input + cache_read + cache_creation;
        let total = normalized_input + output_tokens;

        // Build request with normalized token values
        let request = Request {
            id: format!("claude:{}:{}", session_id, uuid),
            session_id: format!("claude:{}", session_id),
            timestamp: timestamp.to_string(),
            model: model.map(|s| s.to_string()),
            input_tokens: normalized_input,
            output_tokens,
            cache_read_tokens: cache_read,
            cache_creation_tokens: cache_creation,
            reasoning_tokens: 0,
            total_tokens: total,
            duration_ms: None,
        };

        // Get or create session entry
        let tally_session_id = format!("claude:{}", session_id);
        let parsed = sessions_map.entry(tally_session_id.clone()).or_insert_with(|| {
            let source = if is_sidechain || agent_id.is_some() {
                Some("subagent".to_string())
            } else {
                Some("cli".to_string())
            };

            ParsedClaudeSession {
                session: Session {
                    id: tally_session_id,
                    tool: "claude".to_string(),
                    source,
                    parent_session_id: None,
                    model: model.map(|s| s.to_string()),
                    title: None,
                    start_time: timestamp.to_string(),
                    end_time: None,
                    project_path: cwd.map(|s| s.to_string()),
                    project_name: derive_project_name(cwd, None),
                    git_branch: git_branch.map(|s| s.to_string()),
                    git_sha: None,
                    git_origin_url: None,
                    cli_version: version.map(|s| s.to_string()),
                    total_input_tokens: 0,
                    total_output_tokens: 0,
                    total_cache_read_tokens: 0,
                    total_cache_creation_tokens: 0,
                    total_reasoning_tokens: 0,
                    total_tokens: 0,
                    estimated_cost: 0.0,
                },
                requests: Vec::new(),
            }
        });

        // Accumulate totals (using normalized values)
        parsed.session.total_input_tokens += normalized_input;
        parsed.session.total_output_tokens += output_tokens;
        parsed.session.total_cache_read_tokens += cache_read;
        parsed.session.total_cache_creation_tokens += cache_creation;
        parsed.session.total_tokens += total;
        parsed.session.end_time = Some(timestamp.to_string());

        // Update model if newer (later messages may have different model)
        if model.is_some() {
            parsed.session.model = model.map(|s| s.to_string());
        }

        parsed.requests.push(request);
    }

    let sessions: Vec<ParsedClaudeSession> = sessions_map.into_values().collect();
    Ok((sessions, file_size))
}

/// Find all Claude Code JSONL session files
/// Returns Vec<(file_path, encoded_project_dir_name)>
pub fn find_claude_session_files(claude_dir: &Path) -> Vec<(PathBuf, String)> {
    let projects_dir = claude_dir.join("projects");
    let mut files = Vec::new();

    if !projects_dir.exists() {
        return files;
    }

    if let Ok(project_entries) = std::fs::read_dir(&projects_dir) {
        for project_entry in project_entries.flatten() {
            let project_path = project_entry.path();
            if !project_path.is_dir() {
                continue;
            }

            let project_name = project_entry
                .file_name()
                .to_string_lossy()
                .to_string();

            if let Ok(session_entries) = std::fs::read_dir(&project_path) {
                for session_entry in session_entries.flatten() {
                    let session_path = session_entry.path();

                    if session_path.is_file()
                        && session_path.extension().and_then(|e| e.to_str()) == Some("jsonl")
                    {
                        // Direct JSONL file: <session-id>.jsonl
                        files.push((session_path, project_name.clone()));
                    } else if session_path.is_dir() {
                        // Directory session: may contain subagents/
                        // Look for direct JSONL in the directory
                        let session_dir_name = session_entry.file_name().to_string_lossy().to_string();
                        let direct_jsonl = session_path.join(format!("{}.jsonl", session_dir_name));
                        if direct_jsonl.exists() {
                            files.push((direct_jsonl, project_name.clone()));
                        }

                        // Look for subagent JSONL files
                        let subagents_dir = session_path.join("subagents");
                        if subagents_dir.exists() {
                            if let Ok(agent_entries) = std::fs::read_dir(&subagents_dir) {
                                for agent_entry in agent_entries.flatten() {
                                    let agent_path = agent_entry.path();
                                    if agent_path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                                        files.push((agent_path, project_name.clone()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    files
}

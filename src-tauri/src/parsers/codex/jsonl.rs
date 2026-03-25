use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::db::models::Request;

/// Parse a Codex JSONL rollout file for per-request token data
/// Returns a list of Request structs
pub fn parse_codex_jsonl(
    file_path: &Path,
    session_id: &str,
) -> Result<Vec<Request>, String> {
    let file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open {}: {}", file_path.display(), e))?;
    let reader = BufReader::new(file);

    let mut requests = Vec::new();
    let mut prev_total_input: i64 = 0;
    let mut prev_total_output: i64 = 0;
    let mut prev_total_cached: i64 = 0;
    let mut prev_total_reasoning: i64 = 0;
    let mut line_num: usize = 0;

    for line_result in reader.lines() {
        line_num += 1;
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue, // Skip incomplete last line
        };

        if line.trim().is_empty() {
            continue;
        }

        let event: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue, // Skip malformed lines
        };

        let event_type = event.get("type").and_then(|t| t.as_str());
        let timestamp = event.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");

        if event_type == Some("event_msg") {
            if let Some(payload) = event.get("payload") {
                let payload_type = payload.get("type").and_then(|t| t.as_str());
                if payload_type == Some("token_count") {
                    if let Some(info) = payload.get("info") {
                        // Extract total_token_usage (cumulative)
                        if let Some(total_usage) = info.get("total_token_usage") {
                            let curr_input = total_usage.get("input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
                            let curr_output = total_usage.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
                            let curr_cached = total_usage.get("cached_input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
                            let curr_reasoning = total_usage.get("reasoning_output_tokens").and_then(|v| v.as_i64()).unwrap_or(0);

                            // Compute delta (per-request)
                            let delta_input = curr_input - prev_total_input;
                            let delta_output = curr_output - prev_total_output;
                            let delta_cached = curr_cached - prev_total_cached;
                            let delta_reasoning = curr_reasoning - prev_total_reasoning;
                            let delta_total = delta_input + delta_output;

                            if delta_total > 0 {
                                requests.push(Request {
                                    id: format!("codex:{}:{}", session_id, line_num),
                                    session_id: format!("codex:{}", session_id),
                                    timestamp: timestamp.to_string(),
                                    model: None, // Model from session_meta, not per-request
                                    input_tokens: delta_input,
                                    output_tokens: delta_output,
                                    cache_read_tokens: delta_cached,
                                    cache_creation_tokens: 0,
                                    reasoning_tokens: delta_reasoning,
                                    total_tokens: delta_total,
                                    duration_ms: None,
                                });
                            }

                            prev_total_input = curr_input;
                            prev_total_output = curr_output;
                            prev_total_cached = curr_cached;
                            prev_total_reasoning = curr_reasoning;
                        }
                        // Also handle last_token_usage if present
                        else if let Some(last_usage) = info.get("last_token_usage") {
                            let input = last_usage.get("input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
                            let output = last_usage.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
                            let cached = last_usage.get("cached_input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
                            let reasoning = last_usage.get("reasoning_output_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
                            let total = input + output;

                            if total > 0 {
                                requests.push(Request {
                                    id: format!("codex:{}:{}", session_id, line_num),
                                    session_id: format!("codex:{}", session_id),
                                    timestamp: timestamp.to_string(),
                                    model: None,
                                    input_tokens: input,
                                    output_tokens: output,
                                    cache_read_tokens: cached,
                                    cache_creation_tokens: 0,
                                    reasoning_tokens: reasoning,
                                    total_tokens: total,
                                    duration_ms: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(requests)
}

/// Extract session metadata from a Codex JSONL file
/// Returns (session_id, model, model_provider) if found
pub fn extract_session_meta(file_path: &Path) -> Option<(String, Option<String>, Option<String>)> {
    let file = std::fs::File::open(file_path).ok()?;
    let reader = BufReader::new(file);

    for line_result in reader.lines() {
        let line = line_result.ok()?;
        if line.trim().is_empty() {
            continue;
        }

        let event: serde_json::Value = serde_json::from_str(&line).ok()?;

        if event.get("type").and_then(|t| t.as_str()) == Some("session_meta") {
            if let Some(payload) = event.get("payload") {
                let id = payload.get("id").and_then(|v| v.as_str())?.to_string();
                let model = payload.get("model").and_then(|v| v.as_str()).map(|s| s.to_string());
                let provider = payload.get("model_provider").and_then(|v| v.as_str()).map(|s| s.to_string());
                return Some((id, model, provider));
            }
        }
    }

    None
}

/// Scan for all Codex JSONL session files
pub fn find_codex_jsonl_files(codex_dir: &Path) -> Vec<std::path::PathBuf> {
    let sessions_dir = codex_dir.join("sessions");
    let mut files = Vec::new();

    if !sessions_dir.exists() {
        // Also check archived_sessions
        let archived = codex_dir.join("archived_sessions");
        if archived.exists() {
            collect_jsonl_files(&archived, &mut files);
        }
        return files;
    }

    collect_jsonl_files(&sessions_dir, &mut files);

    // Also check archived
    let archived = codex_dir.join("archived_sessions");
    if archived.exists() {
        collect_jsonl_files(&archived, &mut files);
    }

    files
}

fn collect_jsonl_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_jsonl_files(&path, files);
            } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                files.push(path);
            }
        }
    }
}

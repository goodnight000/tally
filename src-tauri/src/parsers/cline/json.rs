use std::collections::HashMap;
use std::path::Path;

use crate::db::models::{Request, Session};

/// Parse all task directories in a Cline-family extension's tasks folder.
/// Returns (sessions, requests, new_mtimes).
pub fn parse_cline_tasks(
    tasks_dir: &Path,
    tool_id: &str,
    prev_mtimes: &HashMap<String, i64>,
) -> Result<(Vec<Session>, Vec<Request>, HashMap<String, i64>), String> {
    let mut all_sessions = Vec::new();
    let mut all_requests = Vec::new();
    let mut new_mtimes: HashMap<String, i64> = HashMap::new();

    let entries = std::fs::read_dir(tasks_dir)
        .map_err(|e| format!("Failed to read tasks dir: {}", e))?;

    for entry in entries.filter_map(|e| e.ok()) {
        let task_dir = entry.path();
        let ui_messages_path = task_dir.join("ui_messages.json");
        if !ui_messages_path.exists() {
            continue;
        }

        let task_id = match task_dir.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Check mtime for incremental sync
        let mtime = std::fs::metadata(&ui_messages_path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        new_mtimes.insert(task_id.clone(), mtime);

        // Skip if not modified since last sync
        if let Some(&prev_mtime) = prev_mtimes.get(&task_id) {
            if mtime <= prev_mtime {
                continue;
            }
        }

        match parse_task_ui_messages(&ui_messages_path, tool_id, &task_id) {
            Ok((session, requests)) => {
                all_sessions.push(session);
                all_requests.extend(requests);
            }
            Err(e) => {
                log::warn!("Failed to parse {}: {}", ui_messages_path.display(), e);
            }
        }
    }

    log::info!(
        "Parsed {} {} sessions, {} requests",
        all_sessions.len(),
        tool_id,
        all_requests.len()
    );

    Ok((all_sessions, all_requests, new_mtimes))
}

/// Parse a single task's ui_messages.json
fn parse_task_ui_messages(
    path: &Path,
    tool_id: &str,
    task_id: &str,
) -> Result<(Session, Vec<Request>), String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Read error: {}", e))?;

    let messages: Vec<serde_json::Value> = serde_json::from_str(&data)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let session_id = format!("{}:{}", tool_id, task_id);
    let mut requests = Vec::new();
    let mut total_input: i64 = 0;
    let mut total_output: i64 = 0;
    let mut total_cache_read: i64 = 0;
    let mut total_cache_write: i64 = 0;
    let mut first_ts: Option<String> = None;
    let mut last_ts: Option<String> = None;
    let mut model: Option<String> = None;
    let mut req_index = 0;

    for msg in &messages {
        let msg_type = msg.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let say = msg.get("say").and_then(|v| v.as_str()).unwrap_or("");

        // Extract timestamp
        let ts_millis = msg.get("ts").and_then(|v| v.as_i64()).unwrap_or(0);
        let ts_iso = if ts_millis > 0 {
            chrono::DateTime::from_timestamp_millis(ts_millis)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default()
        } else {
            String::new()
        };

        if !ts_iso.is_empty() {
            if first_ts.is_none() {
                first_ts = Some(ts_iso.clone());
            }
            last_ts = Some(ts_iso.clone());
        }

        // Look for api_req_started messages with token usage
        if msg_type == "say" && say == "api_req_started" {
            if let Some(text) = msg.get("text").and_then(|v| v.as_str()) {
                if let Ok(usage) = serde_json::from_str::<serde_json::Value>(text) {
                    let tokens_in = usage.get("tokensIn").and_then(|v| v.as_i64()).unwrap_or(0);
                    let tokens_out = usage.get("tokensOut").and_then(|v| v.as_i64()).unwrap_or(0);
                    let cache_writes = usage.get("cacheWrites").and_then(|v| v.as_i64()).unwrap_or(0);
                    let cache_reads = usage.get("cacheReads").and_then(|v| v.as_i64()).unwrap_or(0);

                    // Skip messages with no actual token data (the initial "started" event)
                    if tokens_in == 0 && tokens_out == 0 {
                        continue;
                    }

                    // Extract model if present
                    if let Some(m) = usage.get("modelId").and_then(|v| v.as_str()) {
                        model = Some(m.to_string());
                    }

                    let total = tokens_in + tokens_out;
                    total_input += tokens_in;
                    total_output += tokens_out;
                    total_cache_read += cache_reads;
                    total_cache_write += cache_writes;

                    requests.push(Request {
                        id: format!("{}:{}:{}", tool_id, task_id, req_index),
                        session_id: session_id.clone(),
                        timestamp: ts_iso.clone(),
                        model: model.clone(),
                        input_tokens: tokens_in,
                        output_tokens: tokens_out,
                        cache_read_tokens: cache_reads,
                        cache_creation_tokens: cache_writes,
                        reasoning_tokens: 0,
                        total_tokens: total,
                        duration_ms: None,
                    });

                    req_index += 1;
                }
            }
        }
    }

    let total_tokens = total_input + total_output;

    let session = Session {
        id: session_id,
        tool: tool_id.to_string(),
        source: Some("vscode".to_string()),
        parent_session_id: None,
        model,
        title: None,
        start_time: first_ts.unwrap_or_default(),
        end_time: last_ts,
        project_path: None,
        project_name: None,
        git_branch: None,
        git_sha: None,
        git_origin_url: None,
        cli_version: None,
        total_input_tokens: total_input,
        total_output_tokens: total_output,
        total_cache_read_tokens: total_cache_read,
        total_cache_creation_tokens: total_cache_write,
        total_reasoning_tokens: 0,
        total_tokens,
        estimated_cost: 0.0,
    };

    Ok((session, requests))
}

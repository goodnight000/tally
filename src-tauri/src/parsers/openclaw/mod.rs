use std::collections::HashMap;
use std::io::{BufRead, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db::models::{Request, Session};
use crate::parsers::normalize::unix_ms_to_iso8601;
use crate::parsers::registry::{DetectedSource, SourceSyncResult};

/// Get the OpenClaw data directory
pub fn openclaw_data_dir() -> PathBuf {
    // Check legacy paths too
    let home = dirs::home_dir().expect("Could not determine home directory");
    for name in &[".openclaw", ".clawdbot", ".moldbot", ".moltbot"] {
        let p = home.join(name);
        if p.exists() {
            return p;
        }
    }
    home.join(".openclaw")
}

pub fn registry_detect() -> Option<DetectedSource> {
    let data_dir = openclaw_data_dir();
    let agents_dir = data_dir.join("agents");
    if !agents_dir.exists() {
        return None;
    }

    let mut total_sessions = 0i64;
    if let Ok(entries) = std::fs::read_dir(&agents_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let sessions_dir = entry.path().join("sessions");
            let sessions_file = sessions_dir.join("sessions.json");
            if sessions_file.exists() {
                // Count sessions from the index file
                if let Ok(data) = std::fs::read_to_string(&sessions_file) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
                        if let Some(arr) = val.as_array() {
                            total_sessions += arr.len() as i64;
                        } else if let Some(obj) = val.as_object() {
                            total_sessions += obj.len() as i64;
                        }
                    }
                }
            }
        }
    }

    if total_sessions == 0 {
        return None;
    }

    Some(DetectedSource {
        path: data_dir.to_string_lossy().to_string(),
        session_count: total_sessions,
    })
}

pub fn registry_sync(conn: &Connection, force_full: bool) -> Result<SourceSyncResult, String> {
    let data_dir = openclaw_data_dir();
    let agents_dir = data_dir.join("agents");
    if !agents_dir.exists() {
        return Ok(SourceSyncResult { new_sessions: 0, new_requests: 0 });
    }

    // Read watermark
    let watermark_json = if force_full {
        None
    } else {
        crate::parsers::get_watermark(conn, "openclaw")
    };

    let prev_offsets: HashMap<String, u64> = watermark_json
        .as_ref()
        .and_then(|json| {
            let val: serde_json::Value = serde_json::from_str(json).ok()?;
            let offsets = val.get("file_offsets")?;
            serde_json::from_value(offsets.clone()).ok()
        })
        .unwrap_or_default();

    let mut all_sessions = Vec::new();
    let mut all_requests = Vec::new();
    let mut new_offsets: HashMap<String, u64> = HashMap::new();

    // Iterate all agents
    let agent_entries = std::fs::read_dir(&agents_dir)
        .map_err(|e| format!("Failed to read agents dir: {}", e))?;

    for entry in agent_entries.filter_map(|e| e.ok()) {
        let sessions_dir = entry.path().join("sessions");
        if !sessions_dir.exists() {
            continue;
        }

        // Parse sessions.json for session metadata
        let sessions_file = sessions_dir.join("sessions.json");
        if sessions_file.exists() {
            if let Ok((sessions, _)) = parse_sessions_json(&sessions_file) {
                all_sessions.extend(sessions);
            }
        }

        // Parse JSONL files for per-request data
        if let Ok(entries) = std::fs::read_dir(&sessions_dir) {
            for file_entry in entries.filter_map(|e| e.ok()) {
                let path = file_entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                    let path_str = path.to_string_lossy().to_string();
                    let prev_offset = prev_offsets.get(&path_str).copied().unwrap_or(0);

                    match parse_openclaw_jsonl(&path, prev_offset) {
                        Ok((reqs, new_offset)) => {
                            new_offsets.insert(path_str, new_offset);
                            all_requests.extend(reqs);
                        }
                        Err(e) => {
                            log::warn!("Failed to parse {}: {}", path.display(), e);
                            new_offsets.insert(path_str, prev_offset);
                        }
                    }
                }
            }
        }
    }

    let (s_count, r_count) = crate::parsers::insert_data(conn, &all_sessions, &all_requests);

    // Update watermark
    let new_watermark = serde_json::json!({ "file_offsets": new_offsets });
    crate::parsers::set_watermark(conn, "openclaw", &new_watermark.to_string());

    log::info!(
        "Parsed {} OpenClaw sessions, {} requests",
        all_sessions.len(),
        all_requests.len()
    );

    Ok(SourceSyncResult {
        new_sessions: s_count,
        new_requests: r_count,
    })
}

/// Parse sessions.json — the aggregated session index
fn parse_sessions_json(
    path: &Path,
) -> Result<(Vec<Session>, Vec<String>), String> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| format!("Read error: {}", e))?;

    let val: serde_json::Value = serde_json::from_str(&data)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let mut sessions = Vec::new();
    let mut session_ids = Vec::new();

    // sessions.json can be an array or object of session entries
    let entries: Vec<(&str, &serde_json::Value)> = if let Some(arr) = val.as_array() {
        arr.iter()
            .enumerate()
            .map(|(i, v)| {
                let id = v.get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if id.is_empty() {
                    (Box::leak(format!("session_{}", i).into_boxed_str()) as &str, v)
                } else {
                    (Box::leak(id.to_string().into_boxed_str()) as &str, v)
                }
            })
            .collect()
    } else if let Some(obj) = val.as_object() {
        obj.iter().map(|(k, v)| (k.as_str(), v)).collect()
    } else {
        return Ok((sessions, session_ids));
    };

    for (sess_id, entry) in entries {
        if sess_id.is_empty() {
            continue;
        }

        let input_tokens = entry.get("inputTokens").and_then(|v| v.as_i64()).unwrap_or(0);
        let output_tokens = entry.get("outputTokens").and_then(|v| v.as_i64()).unwrap_or(0);
        let total_tokens = entry.get("totalTokens").and_then(|v| v.as_i64())
            .unwrap_or(input_tokens + output_tokens);
        let cache_read = entry.get("cacheRead").and_then(|v| v.as_i64()).unwrap_or(0);
        let cache_write = entry.get("cacheWrite").and_then(|v| v.as_i64()).unwrap_or(0);
        let model = entry.get("modelOverride")
            .or_else(|| entry.get("model"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let title = entry.get("title")
            .or_else(|| entry.get("name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let provider = entry.get("providerOverride")
            .or_else(|| entry.get("provider"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Timestamps — can be ISO 8601 strings or Unix ms integers
        let start_time = entry.get("createdAt")
            .or_else(|| entry.get("startedAt"))
            .and_then(|v| {
                v.as_str().map(|s| s.to_string())
                    .or_else(|| v.as_i64().map(|ms| unix_ms_to_iso8601(ms)))
            })
            .unwrap_or_default();
        let end_time = entry.get("updatedAt")
            .or_else(|| entry.get("endedAt"))
            .and_then(|v| {
                v.as_str().map(|s| s.to_string())
                    .or_else(|| v.as_i64().map(|ms| unix_ms_to_iso8601(ms)))
            });

        let full_id = format!("openclaw:{}", sess_id);
        session_ids.push(sess_id.to_string());

        sessions.push(Session {
            id: full_id,
            tool: "openclaw".to_string(),
            source: provider,
            parent_session_id: None,
            model,
            title,
            start_time,
            end_time,
            project_path: None,
            project_name: None,
            git_branch: None,
            git_sha: None,
            git_origin_url: None,
            cli_version: None,
            total_input_tokens: input_tokens,
            total_output_tokens: output_tokens,
            total_cache_read_tokens: cache_read,
            total_cache_creation_tokens: cache_write,
            total_reasoning_tokens: 0,
            total_tokens,
            estimated_cost: 0.0,
        });
    }

    Ok((sessions, session_ids))
}

/// Parse an OpenClaw JSONL transcript file for per-request usage
fn parse_openclaw_jsonl(
    path: &Path,
    start_offset: u64,
) -> Result<(Vec<Request>, u64), String> {
    let file = std::fs::File::open(path)
        .map_err(|e| format!("Open error: {}", e))?;

    let file_len = file.metadata()
        .map(|m| m.len())
        .unwrap_or(0);

    if start_offset >= file_len {
        return Ok((Vec::new(), file_len));
    }

    let mut reader = std::io::BufReader::new(file);
    reader.seek(SeekFrom::Start(start_offset))
        .map_err(|e| format!("Seek error: {}", e))?;

    // Derive session_id from filename
    let session_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let mut requests = Vec::new();
    let mut line_num = 0u64;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break,
        };
        line_num += 1;

        if line.trim().is_empty() {
            continue;
        }

        let val: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Look for messages with usage data
        let message = val.get("message").unwrap_or(&val);
        let usage = match message.get("usage") {
            Some(u) => u,
            None => continue,
        };

        let input = usage.get("input")
            .or_else(|| usage.get("inputTokens"))
            .or_else(|| usage.get("input_tokens"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let output = usage.get("output")
            .or_else(|| usage.get("outputTokens"))
            .or_else(|| usage.get("output_tokens"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let cache_read = usage.get("cacheRead")
            .or_else(|| usage.get("cache_read"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let cache_write = usage.get("cacheWrite")
            .or_else(|| usage.get("cache_write"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        if input == 0 && output == 0 {
            continue;
        }

        let model = message.get("model")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let timestamp = val.get("timestamp")
            .or_else(|| val.get("ts"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        requests.push(Request {
            id: format!("openclaw:{}:{}", session_id, start_offset + line_num),
            session_id: format!("openclaw:{}", session_id),
            timestamp,
            model,
            input_tokens: input,
            output_tokens: output,
            cache_read_tokens: cache_read,
            cache_creation_tokens: cache_write,
            reasoning_tokens: 0,
            total_tokens: input + output,
            duration_ms: None,
        });
    }

    Ok((requests, file_len))
}

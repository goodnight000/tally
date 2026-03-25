pub mod jsonl;
pub mod session_index;
pub mod stats_cache;

use std::collections::HashMap;
use std::path::Path;

use rusqlite::Connection;

use crate::db::models::{Request, Session};
use crate::parsers::normalize::decode_claude_project_path;
use crate::parsers::registry::{DetectedSource, SourceSyncResult};

/// Run a full Claude Code sync
/// file_offsets: JSON map of {filepath: byte_offset} from previous sync
/// Returns (sessions, requests, new_file_offsets_json)
pub fn sync_claude(
    claude_dir: &Path,
    file_offsets: Option<&str>,
) -> Result<(Vec<Session>, Vec<Request>, String), String> {
    let prev_offsets: HashMap<String, u64> = file_offsets
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();

    let session_files = jsonl::find_claude_session_files(claude_dir);
    log::info!("Found {} Claude Code JSONL files", session_files.len());

    let mut all_sessions = Vec::new();
    let mut all_requests = Vec::new();
    let mut new_offsets: HashMap<String, u64> = HashMap::new();

    for (file_path, encoded_project) in &session_files {
        let path_str = file_path.to_string_lossy().to_string();
        let prev_offset = prev_offsets.get(&path_str).copied().unwrap_or(0);

        match jsonl::parse_claude_jsonl(file_path, prev_offset) {
            Ok((parsed_sessions, new_offset)) => {
                new_offsets.insert(path_str, new_offset);

                let decoded_project_path = decode_claude_project_path(encoded_project);

                for mut parsed in parsed_sessions {
                    // Enrich session with project path from the directory name
                    if parsed.session.project_path.is_none() {
                        parsed.session.project_path = Some(decoded_project_path.clone());
                    }
                    if parsed.session.project_name.is_none() {
                        parsed.session.project_name = parsed
                            .session
                            .project_path
                            .as_deref()
                            .and_then(|p| {
                                p.trim_end_matches('/')
                                    .rsplit('/')
                                    .next()
                                    .map(|s| s.to_string())
                            });
                    }

                    all_sessions.push(parsed.session);
                    all_requests.extend(parsed.requests);
                }
            }
            Err(e) => {
                log::warn!("Failed to parse {}: {}", file_path.display(), e);
                // Keep previous offset on failure
                new_offsets.insert(path_str, prev_offset);
            }
        }
    }

    let offsets_json = serde_json::to_string(&new_offsets)
        .unwrap_or_else(|_| "{}".to_string());

    log::info!(
        "Parsed {} Claude sessions, {} requests",
        all_sessions.len(),
        all_requests.len()
    );

    Ok((all_sessions, all_requests, offsets_json))
}

/// Check if Claude Code data exists
pub fn detect_claude(claude_dir: &Path) -> Option<(String, i64)> {
    let projects_dir = claude_dir.join("projects");
    if !projects_dir.exists() {
        return None;
    }

    let files = jsonl::find_claude_session_files(claude_dir);
    if files.is_empty() {
        return None;
    }

    Some((projects_dir.to_string_lossy().to_string(), files.len() as i64))
}

// --- Registry interface ---

/// Registry-compatible detect function
pub fn registry_detect() -> Option<DetectedSource> {
    let claude_dir = super::claude_data_dir();
    let (path, count) = detect_claude(&claude_dir)?;
    Some(DetectedSource {
        path,
        session_count: count,
    })
}

/// Registry-compatible sync function
pub fn registry_sync(conn: &Connection, force_full: bool) -> Result<SourceSyncResult, String> {
    let claude_dir = super::claude_data_dir();
    if !claude_dir.exists() {
        return Ok(SourceSyncResult { new_sessions: 0, new_requests: 0 });
    }

    // Read watermark
    let watermark_json = if force_full {
        None
    } else {
        super::get_watermark(conn, "claude")
    };

    let file_offsets: Option<String> = watermark_json.as_ref().and_then(|json| {
        let val: serde_json::Value = serde_json::from_str(json).ok()?;
        let offsets = val.get("file_offsets")?;
        Some(offsets.to_string())
    });

    // Sync
    let (sessions, requests, new_offsets) =
        sync_claude(&claude_dir, file_offsets.as_deref())?;
    let (s_count, r_count) = super::insert_data(conn, &sessions, &requests);

    // Update watermark
    let new_watermark = serde_json::json!({
        "file_offsets": serde_json::from_str::<serde_json::Value>(&new_offsets)
            .unwrap_or(serde_json::Value::Object(Default::default()))
    });
    super::set_watermark(conn, "claude", &new_watermark.to_string());

    Ok(SourceSyncResult {
        new_sessions: s_count,
        new_requests: r_count,
    })
}

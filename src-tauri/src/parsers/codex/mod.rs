pub mod jsonl;
pub mod sqlite;

use std::path::Path;

use rusqlite::Connection;

use crate::db::models::{Request, Session};
use crate::parsers::registry::{DetectedSource, SourceSyncResult};

/// Run a full Codex sync: read SQLite sessions + JSONL per-request data
pub fn sync_codex(
    codex_dir: &Path,
    since_updated_at: Option<i64>,
) -> Result<(Vec<Session>, Vec<Request>), String> {
    // Find the highest version state database
    let (db_path, _version) = sqlite::find_codex_db(codex_dir)
        .ok_or_else(|| "No Codex state database found".to_string())?;

    log::info!("Reading Codex DB: {}", db_path.display());

    // Read sessions from SQLite
    let sessions = sqlite::read_codex_sessions(&db_path, since_updated_at)?;
    log::info!("Found {} Codex sessions", sessions.len());

    // Read per-request data from JSONL files
    let jsonl_files = jsonl::find_codex_jsonl_files(codex_dir);
    log::info!("Found {} Codex JSONL files", jsonl_files.len());

    let mut all_requests = Vec::new();
    for file_path in &jsonl_files {
        // Extract session ID from the JSONL metadata
        if let Some((session_id, _model, _provider)) = jsonl::extract_session_meta(file_path) {
            match jsonl::parse_codex_jsonl(file_path, &session_id) {
                Ok(requests) => all_requests.extend(requests),
                Err(e) => log::warn!("Failed to parse {}: {}", file_path.display(), e),
            }
        }
    }

    log::info!("Parsed {} Codex requests from JSONL", all_requests.len());
    Ok((sessions, all_requests))
}

/// Check if Codex data exists at the given path
pub fn detect_codex(codex_dir: &Path) -> Option<(String, i64)> {
    let (db_path, _version) = sqlite::find_codex_db(codex_dir)?;

    let conn = rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .ok()?;

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM threads WHERE source IN ('cli', 'vscode')",
            [],
            |row| row.get(0),
        )
        .ok()?;

    Some((db_path.to_string_lossy().to_string(), count))
}

/// Wrapper for sqlite::find_codex_db accessible from sync command
pub fn codex_sqlite_find(codex_dir: &Path) -> Option<(std::path::PathBuf, i64)> {
    sqlite::find_codex_db(codex_dir)
}

/// Wrapper for sqlite::get_max_updated_at accessible from sync command
pub fn codex_sqlite_max_updated(db_path: &Path) -> Result<Option<i64>, String> {
    sqlite::get_max_updated_at(db_path)
}

// --- Registry interface ---

/// Registry-compatible detect function
pub fn registry_detect() -> Option<DetectedSource> {
    let codex_dir = super::codex_data_dir();
    let (path, count) = detect_codex(&codex_dir)?;
    Some(DetectedSource {
        path,
        session_count: count,
    })
}

/// Registry-compatible sync function
pub fn registry_sync(conn: &Connection, force_full: bool) -> Result<SourceSyncResult, String> {
    let codex_dir = super::codex_data_dir();
    if !codex_dir.exists() {
        return Ok(SourceSyncResult { new_sessions: 0, new_requests: 0 });
    }

    // Read watermark
    let watermark_json = if force_full {
        None
    } else {
        super::get_watermark(conn, "codex")
    };

    let since_updated_at: Option<i64> = watermark_json.as_ref().and_then(|json| {
        serde_json::from_str::<serde_json::Value>(json)
            .ok()?
            .get("thread_updated_at")?
            .as_i64()
    });

    // Sync
    let (sessions, requests) = sync_codex(&codex_dir, since_updated_at)?;
    let (s_count, r_count) = super::insert_data(conn, &sessions, &requests);

    // Update watermark
    if let Some((db_path, version)) = sqlite::find_codex_db(&codex_dir) {
        if let Ok(Some(max_updated)) = sqlite::get_max_updated_at(&db_path) {
            let new_watermark = serde_json::json!({
                "thread_updated_at": max_updated,
                "db_version": version,
            });
            super::set_watermark(conn, "codex", &new_watermark.to_string());
        }
    }

    Ok(SourceSyncResult {
        new_sessions: s_count,
        new_requests: r_count,
    })
}

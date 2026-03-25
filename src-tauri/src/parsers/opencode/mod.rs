use std::path::PathBuf;

use rusqlite::Connection;

use crate::db::models::{Request, Session};
use crate::parsers::registry::{DetectedSource, SourceSyncResult};

/// Get the OpenCode data directory
pub fn opencode_data_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .expect("Could not determine home directory")
            .join(".local/share/opencode")
    }
    #[cfg(not(target_os = "macos"))]
    {
        dirs::data_local_dir()
            .expect("Could not determine local data directory")
            .join("opencode")
    }
}

fn db_path() -> PathBuf {
    opencode_data_dir().join("opencode.db")
}

pub fn registry_detect() -> Option<DetectedSource> {
    let db = db_path();
    if !db.exists() {
        return None;
    }

    let conn = rusqlite::Connection::open_with_flags(
        &db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .ok()?;

    // Check if sessions table exists (schema varies across versions)
    let has_sessions: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sessions'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .ok()
        .map(|c| c > 0)
        .unwrap_or(false);

    if !has_sessions {
        return None;
    }

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .ok()?;

    Some(DetectedSource {
        path: db.to_string_lossy().to_string(),
        session_count: count,
    })
}

pub fn registry_sync(conn: &Connection, force_full: bool) -> Result<SourceSyncResult, String> {
    let db = db_path();
    if !db.exists() {
        return Ok(SourceSyncResult { new_sessions: 0, new_requests: 0 });
    }

    let oc_conn = rusqlite::Connection::open_with_flags(
        &db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| format!("Failed to open OpenCode DB: {}", e))?;

    // Read watermark
    let watermark_json = if force_full {
        None
    } else {
        crate::parsers::get_watermark(conn, "opencode")
    };

    let max_timestamp: Option<String> = watermark_json.as_ref().and_then(|json| {
        let val: serde_json::Value = serde_json::from_str(json).ok()?;
        val.get("max_timestamp")?.as_str().map(|s| s.to_string())
    });

    let (sessions, requests, new_max_ts) = read_opencode_data(&oc_conn, max_timestamp.as_deref())?;
    let (s_count, r_count) = crate::parsers::insert_data(conn, &sessions, &requests);

    // Update watermark
    if let Some(ts) = new_max_ts {
        let new_watermark = serde_json::json!({ "max_timestamp": ts });
        crate::parsers::set_watermark(conn, "opencode", &new_watermark.to_string());
    }

    Ok(SourceSyncResult {
        new_sessions: s_count,
        new_requests: r_count,
    })
}

/// Read sessions and responses from OpenCode's SQLite database
fn read_opencode_data(
    oc_conn: &rusqlite::Connection,
    since_timestamp: Option<&str>,
) -> Result<(Vec<Session>, Vec<Request>, Option<String>), String> {
    // Discover available tables and columns
    let has_responses = table_exists(oc_conn, "responses");
    let has_messages = table_exists(oc_conn, "messages");

    let mut sessions = Vec::new();
    let mut requests = Vec::new();
    let mut max_ts: Option<String> = None;

    // Read sessions using a helper closure to avoid closure type mismatch
    let session_rows = read_session_rows(oc_conn, since_timestamp)?;

    let mut session_ids = Vec::new();
    for (id, title, model_id, created_at, updated_at) in &session_rows {
        if let Some(ut) = updated_at {
            if max_ts.as_ref().map(|m| ut.as_str() > m.as_str()).unwrap_or(true) {
                max_ts = Some(ut.clone());
            }
        }

        session_ids.push(id.clone());

        sessions.push(Session {
            id: format!("opencode:{}", id),
            tool: "opencode".to_string(),
            source: Some("cli".to_string()),
            parent_session_id: None,
            model: model_id.clone(),
            title: title.clone(),
            start_time: created_at.clone().unwrap_or_default(),
            end_time: updated_at.clone(),
            project_path: None,
            project_name: None,
            git_branch: None,
            git_sha: None,
            git_origin_url: None,
            cli_version: None,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cache_read_tokens: 0,
            total_cache_creation_tokens: 0,
            total_reasoning_tokens: 0,
            total_tokens: 0,
            estimated_cost: 0.0,
        });
    }

    // Read per-request data from responses table if it exists
    if has_responses {
        for session_id in &session_ids {
            if let Ok(mut stmt) = oc_conn.prepare(
                "SELECT id, session_id, model_id, tokens_input, tokens_output, cost, created_at
                 FROM responses WHERE session_id = ?1 ORDER BY created_at",
            ) {
                let rows = stmt
                    .query_map([session_id], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, Option<String>>(2)?,
                            row.get::<_, i64>(3).unwrap_or(0),
                            row.get::<_, i64>(4).unwrap_or(0),
                            row.get::<_, f64>(5).unwrap_or(0.0),
                            row.get::<_, Option<String>>(6)?,
                        ))
                    })
                    .ok();

                if let Some(rows) = rows {
                    for row in rows.filter_map(|r| r.ok()) {
                        let (resp_id, sess_id, model, input, output, _cost, timestamp) = row;
                        requests.push(Request {
                            id: format!("opencode:{}", resp_id),
                            session_id: format!("opencode:{}", sess_id),
                            timestamp: timestamp.unwrap_or_default(),
                            model,
                            input_tokens: input,
                            output_tokens: output,
                            cache_read_tokens: 0,
                            cache_creation_tokens: 0,
                            reasoning_tokens: 0,
                            total_tokens: input + output,
                            duration_ms: None,
                        });
                    }
                }
            }
        }
    } else if has_messages {
        // Fallback: some versions use a messages table with token data
        log::info!("OpenCode: using messages table fallback");
    }

    log::info!(
        "Parsed {} OpenCode sessions, {} requests",
        sessions.len(),
        requests.len()
    );

    Ok((sessions, requests, max_ts))
}

type SessionRow = (String, Option<String>, Option<String>, Option<String>, Option<String>);

fn read_session_rows(
    oc_conn: &rusqlite::Connection,
    since_timestamp: Option<&str>,
) -> Result<Vec<SessionRow>, String> {
    let sql = if since_timestamp.is_some() {
        "SELECT id, title, model_id, created_at, updated_at FROM sessions WHERE updated_at > ?1 ORDER BY created_at"
    } else {
        "SELECT id, title, model_id, created_at, updated_at FROM sessions ORDER BY created_at"
    };

    let mut stmt = oc_conn
        .prepare(sql)
        .map_err(|e| format!("Session query error: {}", e))?;

    let row_mapper = |row: &rusqlite::Row| -> rusqlite::Result<SessionRow> {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        ))
    };

    let rows: Vec<SessionRow> = if let Some(ts) = since_timestamp {
        stmt.query_map([ts], row_mapper)
    } else {
        stmt.query_map([], row_mapper)
    }
    .map_err(|e| format!("Session query error: {}", e))?
    .filter_map(|r| r.ok())
    .collect();

    Ok(rows)
}

fn table_exists(conn: &rusqlite::Connection, name: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
        [name],
        |row| row.get::<_, i64>(0),
    )
    .ok()
    .map(|c| c > 0)
    .unwrap_or(false)
}

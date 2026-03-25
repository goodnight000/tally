pub mod claude;
pub mod cline;
pub mod codex;
pub mod normalize;
pub mod opencode;
pub mod openclaw;
pub mod registry;

use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::db::models::{SourceInfo, SyncResult};

/// Get the Codex data directory
pub fn codex_data_dir() -> PathBuf {
    if let Ok(codex_home) = std::env::var("CODEX_HOME") {
        return PathBuf::from(codex_home);
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        let p = PathBuf::from(xdg).join("codex");
        if p.exists() {
            return p;
        }
    }
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".codex")
}

/// Get the Claude Code data directory
pub fn claude_data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".claude")
}

/// Detect available data sources (registry-driven)
pub fn detect_sources(conn: &Connection) -> Vec<SourceInfo> {
    registry::detect_all_sources(conn)
}

/// Get list of enabled source IDs from preferences + defaults
pub fn get_enabled_source_ids(conn: &Connection) -> Vec<String> {
    detect_sources(conn)
        .into_iter()
        .filter(|s| s.enabled && s.detected)
        .map(|s| s.id)
        .collect()
}

/// Run a full sync from all enabled sources into Tally's database
pub fn sync_all(conn: &Connection, force_full: bool) -> SyncResult {
    let mut result = SyncResult {
        new_sessions: 0,
        new_requests: 0,
        errors: Vec::new(),
    };

    let enabled_ids = get_enabled_source_ids(conn);

    for source_def in registry::SOURCES {
        if !enabled_ids.contains(&source_def.id.to_string()) {
            continue;
        }

        match (source_def.sync)(conn, force_full) {
            Ok(sync_result) => {
                result.new_sessions += sync_result.new_sessions;
                result.new_requests += sync_result.new_requests;
            }
            Err(e) => {
                log::warn!("{} sync error: {}", source_def.display_name, e);
                result.errors.push(format!("{}: {}", source_def.display_name, e));
            }
        }
    }

    // Backfill session token breakdowns from request data where sessions have 0 input/output
    backfill_session_tokens(conn);

    // Auto-populate cost rates for new models
    crate::db::cost_defaults::populate_default_costs(conn);

    result
}

/// Insert sessions and requests into Tally's database
pub fn insert_data(
    conn: &Connection,
    sessions: &[crate::db::models::Session],
    requests: &[crate::db::models::Request],
) -> (i64, i64) {
    let mut new_sessions: i64 = 0;
    let mut new_requests: i64 = 0;

    for session in sessions {
        let result = conn.execute(
            "INSERT OR IGNORE INTO sessions
             (id, tool, source, parent_session_id, model, title, start_time, end_time,
              project_path, project_name, git_branch, git_sha, git_origin_url, cli_version,
              total_input_tokens, total_output_tokens, total_cache_read_tokens,
              total_cache_creation_tokens, total_reasoning_tokens, total_tokens)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
            params![
                session.id, session.tool, session.source, session.parent_session_id,
                session.model, session.title, session.start_time, session.end_time,
                session.project_path, session.project_name, session.git_branch,
                session.git_sha, session.git_origin_url, session.cli_version,
                session.total_input_tokens, session.total_output_tokens,
                session.total_cache_read_tokens, session.total_cache_creation_tokens,
                session.total_reasoning_tokens, session.total_tokens,
            ],
        );
        if let Ok(changes) = result {
            new_sessions += changes as i64;
        }
    }

    for request in requests {
        let result = conn.execute(
            "INSERT OR IGNORE INTO requests
             (id, session_id, timestamp, model, input_tokens, output_tokens,
              cache_read_tokens, cache_creation_tokens, reasoning_tokens, total_tokens, duration_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                request.id, request.session_id, request.timestamp, request.model,
                request.input_tokens, request.output_tokens, request.cache_read_tokens,
                request.cache_creation_tokens, request.reasoning_tokens, request.total_tokens,
                request.duration_ms,
            ],
        );
        if let Ok(changes) = result {
            new_requests += changes as i64;
        }
    }

    (new_sessions, new_requests)
}

// --- Generic watermark API ---

/// Read the watermark JSON blob for a source
pub fn get_watermark(conn: &Connection, source: &str) -> Option<String> {
    conn.query_row(
        "SELECT watermark FROM sync_state WHERE source = ?1",
        params![source],
        |row| row.get(0),
    )
    .ok()
    .flatten()
}

/// Write a watermark JSON blob for a source
pub fn set_watermark(conn: &Connection, source: &str, watermark: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    let _ = conn.execute(
        "INSERT INTO sync_state (source, last_sync_at, watermark)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(source) DO UPDATE SET
            last_sync_at = excluded.last_sync_at,
            watermark = excluded.watermark",
        params![source, now, watermark],
    );
}

// --- Source preferences ---

/// Set a source as enabled or disabled
pub fn set_source_enabled(conn: &Connection, source_id: &str, enabled: bool) {
    let _ = conn.execute(
        "INSERT INTO source_preferences (source_id, enabled, updated_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(source_id) DO UPDATE SET
            enabled = excluded.enabled,
            updated_at = excluded.updated_at",
        params![source_id, enabled as i32],
    );
}

/// Backfill session-level token breakdowns from aggregated request data.
pub fn backfill_session_tokens(conn: &Connection) {
    let affected = conn.execute(
        "UPDATE sessions SET
            total_input_tokens = COALESCE((SELECT SUM(input_tokens) FROM requests WHERE requests.session_id = sessions.id), 0),
            total_output_tokens = COALESCE((SELECT SUM(output_tokens) FROM requests WHERE requests.session_id = sessions.id), 0),
            total_cache_read_tokens = COALESCE((SELECT SUM(cache_read_tokens) FROM requests WHERE requests.session_id = sessions.id), 0),
            total_cache_creation_tokens = COALESCE((SELECT SUM(cache_creation_tokens) FROM requests WHERE requests.session_id = sessions.id), 0),
            total_reasoning_tokens = COALESCE((SELECT SUM(reasoning_tokens) FROM requests WHERE requests.session_id = sessions.id), 0)
         WHERE total_input_tokens = 0 AND total_output_tokens = 0
           AND EXISTS (SELECT 1 FROM requests WHERE requests.session_id = sessions.id AND (input_tokens > 0 OR output_tokens > 0))",
        [],
    ).unwrap_or(0);

    if affected > 0 {
        log::info!("Backfilled token breakdowns for {} sessions", affected);
    }

    // Fix Codex sessions with NULL or "openai" model → gpt-5.3-codex
    let model_fixed = conn.execute(
        "UPDATE sessions SET model = 'gpt-5.3-codex'
         WHERE tool = 'codex' AND (model IS NULL OR model = 'openai' OR model = 'openai (unknown model)')",
        [],
    ).unwrap_or(0);

    if model_fixed > 0 {
        log::info!("Fixed model name for {} Codex sessions", model_fixed);
    }
}

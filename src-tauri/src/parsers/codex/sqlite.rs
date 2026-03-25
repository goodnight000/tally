use rusqlite::{Connection, OpenFlags};
use std::collections::HashSet;
use std::path::Path;

use crate::db::models::Session;
use crate::parsers::normalize::{classify_codex_source, derive_project_name, unix_to_iso8601};

/// Read sessions from a Codex state_*.sqlite database
pub fn read_codex_sessions(
    db_path: &Path,
    since_updated_at: Option<i64>,
) -> Result<Vec<Session>, String> {
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("Failed to open Codex DB: {}", e))?;

    // Discover available columns
    let available_columns = get_available_columns(&conn, "threads")
        .map_err(|e| format!("Failed to read table info: {}", e))?;

    // Build SELECT based on available columns
    let known_columns = [
        "id", "created_at", "updated_at", "source", "cwd", "title",
        "tokens_used", "model", "model_provider", "git_sha", "git_branch",
        "git_origin_url", "cli_version",
    ];

    let select_cols: Vec<&str> = known_columns
        .iter()
        .filter(|c| available_columns.contains(&c.to_string()))
        .copied()
        .collect();

    let mut sql = format!("SELECT {} FROM threads", select_cols.join(", "));

    if let Some(since) = since_updated_at {
        sql.push_str(&format!(" WHERE updated_at > {}", since));
    }

    sql.push_str(" ORDER BY created_at ASC");

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("SQL error: {}", e))?;

    let col_index = |name: &str| -> Option<usize> {
        select_cols.iter().position(|&c| c == name)
    };

    let sessions: Vec<Session> = stmt
        .query_map([], |row| {
            let id: String = row.get(col_index("id").unwrap())?;
            let created_at: i64 = row.get(col_index("created_at").unwrap())?;
            let updated_at: i64 = row.get(col_index("updated_at").unwrap())?;
            let source_raw: String = row.get(col_index("source").unwrap())?;
            let cwd: String = row.get(col_index("cwd").unwrap())?;
            let title: String = row.get(col_index("title").unwrap())?;
            let tokens_used: i64 = row.get(col_index("tokens_used").unwrap())?;

            let model: Option<String> = col_index("model")
                .and_then(|i| row.get::<_, Option<String>>(i).ok())
                .flatten();
            let model_provider: Option<String> = col_index("model_provider")
                .and_then(|i| row.get::<_, Option<String>>(i).ok())
                .flatten();
            // Use model if available, otherwise default to gpt-5.3-codex
            // (the model column was added late; older sessions used gpt-5.3-codex)
            let resolved_model = model.or_else(|| Some("gpt-5.3-codex".to_string()));
            let git_sha: Option<String> = col_index("git_sha")
                .and_then(|i| row.get(i).ok());
            let git_branch: Option<String> = col_index("git_branch")
                .and_then(|i| row.get(i).ok());
            let git_origin_url: Option<String> = col_index("git_origin_url")
                .and_then(|i| row.get(i).ok());
            let cli_version: Option<String> = col_index("cli_version")
                .and_then(|i| row.get(i).ok());

            let (source_type, parent_thread_id) = classify_codex_source(&source_raw);
            let project_name = derive_project_name(Some(&cwd), git_origin_url.as_deref());

            Ok(Session {
                id: format!("codex:{}", id),
                tool: "codex".to_string(),
                source: Some(source_type),
                parent_session_id: parent_thread_id.map(|pid| format!("codex:{}", pid)),
                model: resolved_model,
                title: Some(title),
                start_time: unix_to_iso8601(created_at),
                end_time: Some(unix_to_iso8601(updated_at)),
                project_path: Some(cwd),
                project_name,
                git_branch,
                git_sha,
                git_origin_url,
                cli_version,
                // Codex only gives us total tokens_used, not broken down
                total_input_tokens: 0,
                total_output_tokens: 0,
                total_cache_read_tokens: 0,
                total_cache_creation_tokens: 0,
                total_reasoning_tokens: 0,
                total_tokens: tokens_used,
                estimated_cost: 0.0,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(sessions)
}

/// Get the highest version Codex state database path
pub fn find_codex_db(codex_dir: &Path) -> Option<(std::path::PathBuf, i64)> {
    let mut highest_version: i64 = -1;
    let mut best_path = None;

    if let Ok(entries) = std::fs::read_dir(codex_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("state_") && name.ends_with(".sqlite") {
                if let Some(ver_str) = name.strip_prefix("state_").and_then(|s| s.strip_suffix(".sqlite")) {
                    if let Ok(ver) = ver_str.parse::<i64>() {
                        if ver > highest_version {
                            highest_version = ver;
                            best_path = Some(entry.path());
                        }
                    }
                }
            }
        }
    }

    best_path.map(|p| (p, highest_version))
}

fn get_available_columns(conn: &Connection, table: &str) -> Result<HashSet<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let columns: HashSet<String> = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(columns)
}

/// Get the max updated_at from the threads table (for watermark)
pub fn get_max_updated_at(db_path: &Path) -> Result<Option<i64>, String> {
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("Failed to open Codex DB: {}", e))?;
    let max: Option<i64> = conn
        .query_row("SELECT MAX(updated_at) FROM threads", [], |row| row.get(0))
        .map_err(|e| format!("Query error: {}", e))?;
    Ok(max)
}

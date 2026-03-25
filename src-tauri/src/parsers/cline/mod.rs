pub mod json;

use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::parsers::registry::{DetectedSource, SourceSyncResult};

/// Extension IDs for the Cline family
const CLINE_EXT_ID: &str = "saoudrizwan.claude-dev";
const KILO_EXT_ID: &str = "kilocode.kilo-code";
const ROO_EXT_ID: &str = "rooveterinaryinc.roo-cline";

/// Get the VS Code globalStorage directory (platform-specific)
fn vscode_global_storage_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .expect("Could not determine home directory")
            .join("Library/Application Support/Code/User/globalStorage")
    }
    #[cfg(target_os = "linux")]
    {
        dirs::config_dir()
            .expect("Could not determine config directory")
            .join("Code/User/globalStorage")
    }
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir()
            .expect("Could not determine config directory")
            .join("Code/User/globalStorage")
    }
}

fn extension_data_dir(ext_id: &str) -> PathBuf {
    vscode_global_storage_dir().join(ext_id)
}

pub fn cline_data_dir() -> PathBuf {
    extension_data_dir(CLINE_EXT_ID)
}

pub fn kilo_data_dir() -> PathBuf {
    extension_data_dir(KILO_EXT_ID)
}

pub fn roo_data_dir() -> PathBuf {
    extension_data_dir(ROO_EXT_ID)
}

/// Shared detection logic for Cline-family extensions
fn detect_cline_family(data_dir: &Path) -> Option<DetectedSource> {
    let tasks_dir = data_dir.join("tasks");
    if !tasks_dir.exists() {
        return None;
    }

    let count = std::fs::read_dir(&tasks_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join("ui_messages.json").exists())
        .count();

    if count == 0 {
        return None;
    }

    Some(DetectedSource {
        path: tasks_dir.to_string_lossy().to_string(),
        session_count: count as i64,
    })
}

/// Shared sync logic for Cline-family extensions
fn sync_cline_family(
    conn: &Connection,
    data_dir: &Path,
    tool_id: &str,
    force_full: bool,
) -> Result<SourceSyncResult, String> {
    let tasks_dir = data_dir.join("tasks");
    if !tasks_dir.exists() {
        return Ok(SourceSyncResult { new_sessions: 0, new_requests: 0 });
    }

    // Read watermark: {"task_mtimes": {"task_id": mtime_secs}}
    let watermark_json = if force_full {
        None
    } else {
        crate::parsers::get_watermark(conn, tool_id)
    };

    let prev_mtimes: std::collections::HashMap<String, i64> = watermark_json
        .as_ref()
        .and_then(|json| {
            let val: serde_json::Value = serde_json::from_str(json).ok()?;
            let mtimes = val.get("task_mtimes")?;
            serde_json::from_value(mtimes.clone()).ok()
        })
        .unwrap_or_default();

    let (sessions, requests, new_mtimes) =
        json::parse_cline_tasks(&tasks_dir, tool_id, &prev_mtimes)?;

    let (s_count, r_count) = crate::parsers::insert_data(conn, &sessions, &requests);

    // Update watermark
    let new_watermark = serde_json::json!({ "task_mtimes": new_mtimes });
    crate::parsers::set_watermark(conn, tool_id, &new_watermark.to_string());

    Ok(SourceSyncResult {
        new_sessions: s_count,
        new_requests: r_count,
    })
}

// --- Registry entry points ---

pub fn detect_cline() -> Option<DetectedSource> {
    detect_cline_family(&cline_data_dir())
}

pub fn detect_kilo() -> Option<DetectedSource> {
    detect_cline_family(&kilo_data_dir())
}

pub fn detect_roo() -> Option<DetectedSource> {
    detect_cline_family(&roo_data_dir())
}

pub fn sync_cline(conn: &Connection, force_full: bool) -> Result<SourceSyncResult, String> {
    sync_cline_family(conn, &cline_data_dir(), "cline", force_full)
}

pub fn sync_kilo(conn: &Connection, force_full: bool) -> Result<SourceSyncResult, String> {
    sync_cline_family(conn, &kilo_data_dir(), "kilo", force_full)
}

pub fn sync_roo(conn: &Connection, force_full: bool) -> Result<SourceSyncResult, String> {
    sync_cline_family(conn, &roo_data_dir(), "roo", force_full)
}

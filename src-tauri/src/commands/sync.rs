use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::db::models::{SourceInfo, SyncResult};
use crate::parsers;

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub read_db: Arc<Mutex<Connection>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    pub phase: String,       // "{source_id}_sync", "done"
    pub message: String,     // Human-readable status
    pub sessions_so_far: i64,
    pub requests_so_far: i64,
}

#[tauri::command]
pub fn detect_sources(state: State<'_, AppState>) -> Result<Vec<SourceInfo>, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(parsers::detect_sources(&conn))
}

/// Async sync that runs heavy work on a background thread and emits progress events.
#[tauri::command]
pub async fn sync_data(
    app: AppHandle,
    state: State<'_, AppState>,
    force_full: bool,
) -> Result<(), String> {
    let db = Arc::clone(&state.db);

    tauri::async_runtime::spawn(async move {
        let result = run_sync_with_progress(&app, &db, force_full);
        let _ = app.emit("sync-complete", result);
    });

    Ok(())
}

/// Synchronous sync that blocks — used only by commands that need the result immediately
pub fn sync_data_blocking(db: &Arc<Mutex<Connection>>, force_full: bool) -> SyncResult {
    let conn = db.lock().unwrap();
    parsers::sync_all(&conn, force_full)
}

fn run_sync_with_progress(
    app: &AppHandle,
    db: &Arc<Mutex<Connection>>,
    force_full: bool,
) -> SyncResult {
    let mut result = SyncResult {
        new_sessions: 0,
        new_requests: 0,
        errors: Vec::new(),
    };

    // Get enabled source IDs
    let enabled_ids = {
        let conn = db.lock().unwrap();
        parsers::get_enabled_source_ids(&conn)
    };

    // Sync each enabled source
    for source_def in parsers::registry::SOURCES {
        if !enabled_ids.contains(&source_def.id.to_string()) {
            continue;
        }

        // Emit progress
        let _ = app.emit("sync-progress", SyncProgress {
            phase: format!("{}_sync", source_def.id),
            message: format!("Syncing {} data...", source_def.display_name),
            sessions_so_far: result.new_sessions,
            requests_so_far: result.new_requests,
        });

        // Each source's registry_sync handles its own read + write + watermark
        let conn = db.lock().unwrap();
        match (source_def.sync)(&conn, force_full) {
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

    // Backfill + auto-populate costs
    {
        let conn = db.lock().unwrap();
        parsers::backfill_session_tokens(&conn);
        crate::db::cost_defaults::populate_default_costs(&conn);
    }

    // Done
    let _ = app.emit("sync-progress", SyncProgress {
        phase: "done".to_string(),
        message: format!(
            "Imported {} sessions, {} requests",
            result.new_sessions, result.new_requests
        ),
        sessions_so_far: result.new_sessions,
        requests_so_far: result.new_requests,
    });

    result
}

#[tauri::command]
pub fn get_sync_status(
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::models::SyncState>, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut stmt = conn
        .prepare("SELECT source, last_sync_at, watermark FROM sync_state")
        .map_err(|e| format!("SQL error: {}", e))?;

    let states: Vec<crate::db::models::SyncState> = stmt
        .query_map([], |row| {
            Ok(crate::db::models::SyncState {
                source: row.get(0)?,
                last_sync_at: row.get(1)?,
                watermark: row.get(2)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(states)
}

#[tauri::command]
pub fn set_source_enabled(
    state: State<'_, AppState>,
    source_id: String,
    enabled: bool,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    parsers::set_source_enabled(&conn, &source_id, enabled);
    Ok(())
}

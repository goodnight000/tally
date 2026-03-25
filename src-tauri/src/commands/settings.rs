use tauri::State;

use crate::commands::sync::AppState;
use crate::db::models::*;
use crate::db::queries;

#[tauri::command]
pub fn get_cost_rates(state: State<'_, AppState>) -> Result<Vec<CostRate>, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;
    queries::get_cost_rates(&conn).map_err(|e| format!("Query error: {}", e))
}

#[tauri::command]
pub fn update_cost_rate(state: State<'_, AppState>, rate: CostRate) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;
    queries::upsert_cost_rate(&conn, &rate).map_err(|e| format!("Query error: {}", e))
}

#[tauri::command]
pub fn get_diagnostics(state: State<'_, AppState>) -> Result<Diagnostics, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;

    let total_sessions: i64 = conn
        .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .unwrap_or(0);

    let total_requests: i64 = conn
        .query_row("SELECT COUNT(*) FROM requests", [], |row| row.get(0))
        .unwrap_or(0);

    let db_path = crate::db::db_path();
    let db_size = std::fs::metadata(&db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let source_sync_times: Vec<SourceSyncTime> = conn
        .prepare("SELECT source, last_sync_at FROM sync_state")
        .ok()
        .map(|mut stmt| {
            stmt.query_map([], |row| {
                Ok(SourceSyncTime {
                    source_id: row.get(0)?,
                    last_sync_at: row.get(1)?,
                })
            })
            .ok()
            .map(|rows| rows.filter_map(|r| r.ok()).collect())
            .unwrap_or_default()
        })
        .unwrap_or_default();

    Ok(Diagnostics {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        db_size_bytes: db_size,
        total_sessions,
        total_requests,
        source_sync_times,
    })
}

#[tauri::command]
pub fn export_data(
    state: State<'_, AppState>,
    format: String,
) -> Result<String, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;

    let sessions_filters = SessionFilters {
        tool: None, model: None, project: None, source: None,
        start_date: None, end_date: None, token_min: None, token_max: None,
        search: None, sort_by: None, sort_dir: None,
        page: Some(1), page_size: Some(100000),
    };
    let page = queries::get_sessions(&conn, &sessions_filters)
        .map_err(|e| format!("Query error: {}", e))?;

    let export_dir = crate::db::tally_data_dir();
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

    match format.as_str() {
        "json" => {
            let path = export_dir.join(format!("tally_export_{}.json", timestamp));
            let json = serde_json::to_string_pretty(&page.sessions)
                .map_err(|e| format!("Serialize error: {}", e))?;
            std::fs::write(&path, json)
                .map_err(|e| format!("Write error: {}", e))?;
            Ok(path.to_string_lossy().to_string())
        }
        "csv" => {
            let path = export_dir.join(format!("tally_export_{}.csv", timestamp));
            let mut csv = String::from("id,tool,source,model,start_time,project_name,total_input_tokens,total_output_tokens,total_cache_read_tokens,total_cache_creation_tokens,total_reasoning_tokens,total_tokens\n");
            for s in &page.sessions {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{}\n",
                    s.id,
                    s.tool,
                    s.source.as_deref().unwrap_or(""),
                    s.model.as_deref().unwrap_or(""),
                    s.start_time,
                    s.project_name.as_deref().unwrap_or(""),
                    s.total_input_tokens,
                    s.total_output_tokens,
                    s.total_cache_read_tokens,
                    s.total_cache_creation_tokens,
                    s.total_reasoning_tokens,
                    s.total_tokens,
                ));
            }
            std::fs::write(&path, csv)
                .map_err(|e| format!("Write error: {}", e))?;
            Ok(path.to_string_lossy().to_string())
        }
        _ => Err(format!("Unsupported format: {}", format)),
    }
}

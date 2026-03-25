use tauri::State;

use crate::commands::sync::AppState;
use crate::db::models::*;
use crate::db::queries;

#[tauri::command]
pub fn get_sessions(
    state: State<'_, AppState>,
    filters: SessionFilters,
) -> Result<SessionPage, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;
    queries::get_sessions(&conn, &filters).map_err(|e| format!("Query error: {}", e))
}

#[tauri::command]
pub fn get_session_detail(
    state: State<'_, AppState>,
    id: String,
) -> Result<SessionDetail, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;
    queries::get_session_detail(&conn, &id).map_err(|e| format!("Query error: {}", e))
}

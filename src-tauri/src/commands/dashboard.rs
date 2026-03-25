use tauri::State;

use crate::commands::sync::AppState;
use crate::db::models::*;
use crate::db::queries;

#[tauri::command]
pub fn get_dashboard_stats(
    state: State<'_, AppState>,
    tool: Option<String>,
    start: Option<String>,
    end: Option<String>,
) -> Result<DashboardStats, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;
    queries::get_dashboard_stats(&conn, tool.as_deref(), start.as_deref(), end.as_deref())
        .map_err(|e| format!("Query error: {}", e))
}

#[tauri::command]
pub fn get_daily_usage(
    state: State<'_, AppState>,
    tool: Option<String>,
    start: Option<String>,
    end: Option<String>,
) -> Result<Vec<DailyStat>, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;
    queries::get_daily_usage(&conn, tool.as_deref(), start.as_deref(), end.as_deref())
        .map_err(|e| format!("Query error: {}", e))
}

#[tauri::command]
pub fn get_model_breakdown(
    state: State<'_, AppState>,
    tool: Option<String>,
    start: Option<String>,
    end: Option<String>,
) -> Result<Vec<ModelBreakdown>, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;
    queries::get_model_breakdown(&conn, tool.as_deref(), start.as_deref(), end.as_deref())
        .map_err(|e| format!("Query error: {}", e))
}

#[tauri::command]
pub fn get_project_breakdown(
    state: State<'_, AppState>,
    tool: Option<String>,
    start: Option<String>,
    end: Option<String>,
) -> Result<Vec<ProjectSummary>, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;
    queries::get_project_breakdown(&conn, tool.as_deref(), start.as_deref(), end.as_deref())
        .map_err(|e| format!("Query error: {}", e))
}

#[tauri::command]
pub fn get_heatmap_data(
    state: State<'_, AppState>,
    tool: Option<String>,
) -> Result<Vec<HeatmapEntry>, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;
    queries::get_heatmap_data(&conn, tool.as_deref())
        .map_err(|e| format!("Query error: {}", e))
}

#[tauri::command]
pub fn get_top_sessions(
    state: State<'_, AppState>,
    tool: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<Session>, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;
    queries::get_top_sessions(&conn, tool.as_deref(), limit.unwrap_or(10))
        .map_err(|e| format!("Query error: {}", e))
}

#[tauri::command]
pub fn get_hourly_usage(
    state: State<'_, AppState>,
    tool: Option<String>,
    date: String,
) -> Result<Vec<DailyStat>, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;
    queries::get_hourly_usage(&conn, tool.as_deref(), &date)
        .map_err(|e| format!("Query error: {}", e))
}

#[tauri::command]
pub fn get_daily_activity(
    state: State<'_, AppState>,
    tool: Option<String>,
    days: Option<i64>,
) -> Result<Vec<DailyActivity>, String> {
    let conn = state.read_db.lock().map_err(|e| format!("Lock error: {}", e))?;
    queries::get_daily_activity(&conn, tool.as_deref(), days.unwrap_or(30))
        .map_err(|e| format!("Query error: {}", e))
}

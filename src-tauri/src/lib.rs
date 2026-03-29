pub mod commands;
pub mod db;
pub mod parsers;

use commands::sync::AppState;
use std::sync::{Arc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    let conn = db::init_db().expect("Failed to initialize Tally database");
    let read_conn = db::init_read_db().expect("Failed to open read-only database connection");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            db: Arc::new(Mutex::new(conn)),
            read_db: Arc::new(Mutex::new(read_conn)),
        })
        .invoke_handler(tauri::generate_handler![
            // Sync
            commands::sync::detect_sources,
            commands::sync::sync_data,
            commands::sync::get_sync_status,
            commands::sync::set_source_enabled,
            // Dashboard
            commands::dashboard::get_dashboard_stats,
            commands::dashboard::get_daily_usage,
            commands::dashboard::get_model_breakdown,
            commands::dashboard::get_project_breakdown,
            commands::dashboard::get_heatmap_data,
            commands::dashboard::get_top_sessions,
            commands::dashboard::get_hourly_usage,
            commands::dashboard::get_daily_activity,
            // Sessions
            commands::sessions::get_sessions,
            commands::sessions::get_session_detail,
            // Settings
            commands::settings::get_cost_rates,
            commands::settings::update_cost_rate,
            commands::settings::get_diagnostics,
            commands::settings::export_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

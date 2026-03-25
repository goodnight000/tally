use std::path::PathBuf;

use rusqlite::Connection;

use crate::db::models::SourceInfo;

/// Metadata returned when a source is detected on disk
pub struct DetectedSource {
    pub path: String,
    pub session_count: i64,
}

/// Result of syncing a single source
pub struct SourceSyncResult {
    pub new_sessions: i64,
    pub new_requests: i64,
}

/// Definition of a data source that Tally can track
pub struct SourceDef {
    pub id: &'static str,
    pub display_name: &'static str,
    pub color: &'static str,
    pub icon_name: &'static str,
    /// Returns the data directory for this source
    pub data_dir: fn() -> PathBuf,
    /// Detect if this source is installed; returns path + session count
    pub detect: fn() -> Option<DetectedSource>,
    /// Sync this source into Tally's DB. Reads its own watermark, inserts data, updates watermark.
    pub sync: fn(&Connection, bool) -> Result<SourceSyncResult, String>,
}

/// All supported sources, in display order
pub static SOURCES: &[SourceDef] = &[
    SourceDef {
        id: "claude",
        display_name: "Claude Code",
        color: "#CC785C",
        icon_name: "claude",
        data_dir: super::claude_data_dir,
        detect: super::claude::registry_detect,
        sync: super::claude::registry_sync,
    },
    SourceDef {
        id: "codex",
        display_name: "Codex CLI",
        color: "#7B8CEA",
        icon_name: "codex",
        data_dir: super::codex_data_dir,
        detect: super::codex::registry_detect,
        sync: super::codex::registry_sync,
    },
    SourceDef {
        id: "cline",
        display_name: "Cline",
        color: "#F59E0B",
        icon_name: "cline",
        data_dir: super::cline::cline_data_dir,
        detect: super::cline::detect_cline,
        sync: super::cline::sync_cline,
    },
    SourceDef {
        id: "kilo",
        display_name: "Kilo Code",
        color: "#10B981",
        icon_name: "kilo",
        data_dir: super::cline::kilo_data_dir,
        detect: super::cline::detect_kilo,
        sync: super::cline::sync_kilo,
    },
    SourceDef {
        id: "roo",
        display_name: "Roo Code",
        color: "#8B5CF6",
        icon_name: "roo",
        data_dir: super::cline::roo_data_dir,
        detect: super::cline::detect_roo,
        sync: super::cline::sync_roo,
    },
    SourceDef {
        id: "opencode",
        display_name: "OpenCode",
        color: "#06B6D4",
        icon_name: "opencode",
        data_dir: super::opencode::opencode_data_dir,
        detect: super::opencode::registry_detect,
        sync: super::opencode::registry_sync,
    },
    SourceDef {
        id: "openclaw",
        display_name: "OpenClaw",
        color: "#EF4444",
        icon_name: "openclaw",
        data_dir: super::openclaw::openclaw_data_dir,
        detect: super::openclaw::registry_detect,
        sync: super::openclaw::registry_sync,
    },
];

/// Build SourceInfo list by running detection and joining with preferences
pub fn detect_all_sources(conn: &Connection) -> Vec<SourceInfo> {
    // Load preferences from DB
    let prefs = load_preferences(conn);

    SOURCES
        .iter()
        .map(|def| {
            let detected = (def.detect)();
            let enabled = prefs
                .iter()
                .find(|p| p.source_id == def.id)
                .map(|p| p.enabled)
                .unwrap_or(detected.is_some()); // default: enabled if detected

            SourceInfo {
                id: def.id.to_string(),
                display_name: def.display_name.to_string(),
                detected: detected.is_some(),
                path: detected.as_ref().map(|d| d.path.clone()),
                session_count: detected.as_ref().map(|d| d.session_count).unwrap_or(0),
                enabled,
                color: def.color.to_string(),
                icon_name: def.icon_name.to_string(),
            }
        })
        .collect()
}

fn load_preferences(conn: &Connection) -> Vec<crate::db::models::SourcePreference> {
    let mut stmt = match conn.prepare("SELECT source_id, enabled FROM source_preferences") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    stmt.query_map([], |row| {
        Ok(crate::db::models::SourcePreference {
            source_id: row.get(0)?,
            enabled: row.get::<_, i32>(1)? != 0,
        })
    })
    .ok()
    .map(|rows| rows.filter_map(|r| r.ok()).collect())
    .unwrap_or_default()
}

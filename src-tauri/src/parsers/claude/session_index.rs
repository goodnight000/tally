use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionIndexEntry {
    pub pid: i64,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub cwd: String,
    #[serde(rename = "startedAt")]
    pub started_at: i64, // Unix timestamp in milliseconds
}

/// Read all session index entries from ~/.claude/sessions/
pub fn read_session_index(claude_dir: &Path) -> Vec<SessionIndexEntry> {
    let sessions_dir = claude_dir.join("sessions");
    let mut entries = Vec::new();

    if !sessions_dir.exists() {
        return entries;
    }

    if let Ok(dir_entries) = std::fs::read_dir(&sessions_dir) {
        for entry in dir_entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(index_entry) = serde_json::from_str::<SessionIndexEntry>(&content) {
                        entries.push(index_entry);
                    }
                }
            }
        }
    }

    entries
}

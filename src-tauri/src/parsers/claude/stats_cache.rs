use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsCache {
    pub version: Option<i32>,
    #[serde(rename = "lastComputedDate")]
    pub last_computed_date: Option<String>,
    #[serde(rename = "dailyActivity", default)]
    pub daily_activity: Vec<DailyActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivity {
    pub date: String,
    #[serde(rename = "messageCount", default)]
    pub message_count: i64,
    #[serde(rename = "sessionCount", default)]
    pub session_count: i64,
    #[serde(rename = "toolCallCount", default)]
    pub tool_call_count: i64,
}

/// Parse the Claude Code stats-cache.json for quick initial render
pub fn parse_stats_cache(claude_dir: &Path) -> Option<StatsCache> {
    let path = claude_dir.join("stats-cache.json");
    if !path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

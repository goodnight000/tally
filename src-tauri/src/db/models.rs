use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub tool: String,
    pub source: Option<String>,
    pub parent_session_id: Option<String>,
    pub model: Option<String>,
    pub title: Option<String>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub project_path: Option<String>,
    pub project_name: Option<String>,
    pub git_branch: Option<String>,
    pub git_sha: Option<String>,
    pub git_origin_url: Option<String>,
    pub cli_version: Option<String>,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cache_read_tokens: i64,
    pub total_cache_creation_tokens: i64,
    pub total_reasoning_tokens: i64,
    pub total_tokens: i64,
    #[serde(default)]
    pub estimated_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub session_id: String,
    pub timestamp: String,
    pub model: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub reasoning_tokens: i64,
    pub total_tokens: i64,
    pub duration_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRate {
    pub model: String,
    pub input_per_million: Option<f64>,
    pub output_per_million: Option<f64>,
    pub cache_read_per_million: Option<f64>,
    pub cache_creation_per_million: Option<f64>,
    pub effective_from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub source: String,
    pub last_sync_at: String,
    pub watermark: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStat {
    pub date: String,
    pub tool: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub reasoning_tokens: i64,
    pub total_tokens: i64,
    pub session_count: i64,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBreakdown {
    pub model: String,
    pub tool: String,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub project_name: String,
    pub total_tokens: i64,
    pub session_count: i64,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub streak: i32,
    pub tokens_today: i64,
    pub sessions_today: i64,
    pub total_tokens: i64,
    pub total_sessions: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cache_read_tokens: i64,
    pub total_cache_creation_tokens: i64,
    pub total_reasoning_tokens: i64,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapEntry {
    pub day_of_week: i32,
    pub hour: i32,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivity {
    pub date: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFilters {
    pub tool: Option<String>,
    pub model: Option<Vec<String>>,
    pub project: Option<Vec<String>>,
    pub source: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub token_min: Option<i64>,
    pub token_max: Option<i64>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPage {
    pub sessions: Vec<Session>,
    pub total_count: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetail {
    pub session: Session,
    pub requests: Vec<Request>,
    pub children: Vec<Session>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    pub id: String,
    pub display_name: String,
    pub detected: bool,
    pub path: Option<String>,
    pub session_count: i64,
    pub enabled: bool,
    pub color: String,
    pub icon_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePreference {
    pub source_id: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub new_sessions: i64,
    pub new_requests: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSyncTime {
    pub source_id: String,
    pub last_sync_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostics {
    pub app_version: String,
    pub db_size_bytes: u64,
    pub total_sessions: i64,
    pub total_requests: i64,
    pub source_sync_times: Vec<SourceSyncTime>,
}

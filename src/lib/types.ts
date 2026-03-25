export interface Session {
  id: string;
  tool: string;
  source: string | null;
  parent_session_id: string | null;
  model: string | null;
  title: string | null;
  start_time: string;
  end_time: string | null;
  project_path: string | null;
  project_name: string | null;
  git_branch: string | null;
  git_sha: string | null;
  git_origin_url: string | null;
  cli_version: string | null;
  total_input_tokens: number;
  total_output_tokens: number;
  total_cache_read_tokens: number;
  total_cache_creation_tokens: number;
  total_reasoning_tokens: number;
  total_tokens: number;
  estimated_cost: number;
}

export interface Request {
  id: string;
  session_id: string;
  timestamp: string;
  model: string | null;
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cache_creation_tokens: number;
  reasoning_tokens: number;
  total_tokens: number;
  duration_ms: number | null;
}

export interface CostRate {
  model: string;
  input_per_million: number | null;
  output_per_million: number | null;
  cache_read_per_million: number | null;
  cache_creation_per_million: number | null;
  effective_from: string | null;
}

export interface DashboardStats {
  streak: number;
  tokens_today: number;
  sessions_today: number;
  total_tokens: number;
  total_sessions: number;
  total_input_tokens: number;
  total_output_tokens: number;
  total_cache_read_tokens: number;
  total_cache_creation_tokens: number;
  total_reasoning_tokens: number;
  estimated_cost: number;
}

export interface DailyStat {
  date: string;
  tool: string;
  input_tokens: number;
  output_tokens: number;
  cache_read_tokens: number;
  cache_creation_tokens: number;
  reasoning_tokens: number;
  total_tokens: number;
  session_count: number;
  estimated_cost: number;
}

export interface ModelBreakdown {
  model: string;
  tool: string;
  total_tokens: number;
  input_tokens: number;
  output_tokens: number;
  estimated_cost: number;
}

export interface ProjectSummary {
  project_name: string;
  total_tokens: number;
  session_count: number;
  estimated_cost: number;
}

export interface DailyActivity {
  date: string;
  count: number;
}

export interface HeatmapEntry {
  day_of_week: number;
  hour: number;
  count: number;
}

export interface SessionFilters {
  tool?: string;
  model?: string[];
  project?: string[];
  source?: string;
  start_date?: string;
  end_date?: string;
  token_min?: number;
  token_max?: number;
  search?: string;
  sort_by?: string;
  sort_dir?: string;
  page?: number;
  page_size?: number;
}

export interface SessionPage {
  sessions: Session[];
  total_count: number;
  page: number;
  page_size: number;
}

export interface SessionDetail {
  session: Session;
  requests: Request[];
  children: Session[];
}

export interface SourceInfo {
  id: string;
  display_name: string;
  detected: boolean;
  path: string | null;
  session_count: number;
  enabled: boolean;
  color: string;
  icon_name: string;
}

export interface SyncResult {
  new_sessions: number;
  new_requests: number;
  errors: string[];
}

export interface SyncState {
  source: string;
  last_sync_at: string;
  watermark: string | null;
}

export interface SourceSyncTime {
  source_id: string;
  last_sync_at: string;
}

export interface Diagnostics {
  app_version: string;
  db_size_bytes: number;
  total_sessions: number;
  total_requests: number;
  source_sync_times: SourceSyncTime[];
}

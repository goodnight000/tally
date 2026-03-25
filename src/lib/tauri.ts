import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  CostRate,
  DailyActivity,
  DailyStat,
  DashboardStats,
  Diagnostics,
  HeatmapEntry,
  ModelBreakdown,
  ProjectSummary,
  Session,
  SessionDetail,
  SessionFilters,
  SessionPage,
  SourceInfo,
  SyncResult,
  SyncState,
} from "./types";

// Sync
export const detectSources = () => invoke<SourceInfo[]>("detect_sources");

/** Starts async sync — returns immediately. Listen for events to track progress. */
export const syncData = (forceFull: boolean) =>
  invoke<void>("sync_data", { forceFull });

export const getSyncStatus = () => invoke<SyncState[]>("get_sync_status");

export interface SyncProgress {
  phase: string;
  message: string;
  sessions_so_far: number;
  requests_so_far: number;
}

/** Listen for sync progress events from the Rust backend */
export function onSyncProgress(callback: (progress: SyncProgress) => void) {
  return listen<SyncProgress>("sync-progress", (event) => {
    callback(event.payload);
  });
}

/** Listen for sync completion event */
export function onSyncComplete(callback: (result: SyncResult) => void) {
  return listen<SyncResult>("sync-complete", (event) => {
    callback(event.payload);
  });
}

// Dashboard
export const getDashboardStats = (params: {
  tool?: string;
  start?: string;
  end?: string;
}) => invoke<DashboardStats>("get_dashboard_stats", params);

export const getDailyUsage = (params: {
  tool?: string;
  start?: string;
  end?: string;
}) => invoke<DailyStat[]>("get_daily_usage", params);

export const getModelBreakdown = (params: {
  tool?: string;
  start?: string;
  end?: string;
}) => invoke<ModelBreakdown[]>("get_model_breakdown", params);

export const getProjectBreakdown = (params: {
  tool?: string;
  start?: string;
  end?: string;
}) => invoke<ProjectSummary[]>("get_project_breakdown", params);

export const getHeatmapData = (params: { tool?: string }) =>
  invoke<HeatmapEntry[]>("get_heatmap_data", params);

export const getTopSessions = (params: { tool?: string; limit?: number }) =>
  invoke<Session[]>("get_top_sessions", params);

export const getHourlyUsage = (params: { tool?: string; date: string }) =>
  invoke<DailyStat[]>("get_hourly_usage", params);

export const getDailyActivity = (params: { tool?: string; days?: number }) =>
  invoke<DailyActivity[]>("get_daily_activity", params);

// Sessions
export const getSessions = (filters: SessionFilters) =>
  invoke<SessionPage>("get_sessions", { filters });

export const getSessionDetail = (id: string) =>
  invoke<SessionDetail>("get_session_detail", { id });

// Settings
export const getCostRates = () => invoke<CostRate[]>("get_cost_rates");
export const updateCostRate = (rate: CostRate) =>
  invoke<void>("update_cost_rate", { rate });
export const getDiagnostics = () => invoke<Diagnostics>("get_diagnostics");
export const exportData = (format: string) =>
  invoke<string>("export_data", { format });

// Source preferences
export const setSourceEnabled = (sourceId: string, enabled: boolean) =>
  invoke<void>("set_source_enabled", { sourceId, enabled });

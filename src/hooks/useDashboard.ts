import { useState, useEffect, useCallback } from "react";
import {
  getDashboardStats,
  getDailyUsage,
  getModelBreakdown,
  getProjectBreakdown,
  getHeatmapData,
  getTopSessions,
  getHourlyUsage,
  getDailyActivity,
} from "../lib/tauri";
import { today } from "../lib/format";
import type {
  DashboardStats,
  DailyStat,
  DailyActivity,
  ModelBreakdown,
  ProjectSummary,
  HeatmapEntry,
  Session,
} from "../lib/types";

interface DashboardData {
  stats: DashboardStats | null;
  dailyUsage: DailyStat[];
  hourlyUsage: DailyStat[];
  dailyActivity: DailyActivity[];
  modelBreakdown: ModelBreakdown[];
  projectBreakdown: ProjectSummary[];
  heatmap: HeatmapEntry[];
  topSessions: Session[];
  loading: boolean;
  error: string | null;
  isToday: boolean;
}

export function useDashboard(
  toolFilter?: string,
  dateRange?: { start: string; end: string } | null
) {
  const [data, setData] = useState<DashboardData>({
    stats: null,
    dailyUsage: [],
    hourlyUsage: [],
    dailyActivity: [],
    modelBreakdown: [],
    projectBreakdown: [],
    heatmap: [],
    topSessions: [],
    loading: true,
    error: null,
    isToday: false,
  });

  const fetchData = useCallback(async () => {
    setData((prev) => ({ ...prev, loading: prev.stats === null, error: null }));
    try {
      const params = {
        tool: toolFilter,
        start: dateRange?.start,
        end: dateRange?.end,
      };

      // Detect if "Today" is selected (same start and end date)
      const todayDate = today();
      const isToday =
        dateRange != null &&
        dateRange.start === todayDate;

      const [stats, dailyUsage, modelBreakdown, projectBreakdown, heatmap, topSessions, dailyActivity] =
        await Promise.all([
          getDashboardStats(params),
          getDailyUsage(params),
          getModelBreakdown(params),
          getProjectBreakdown(params),
          getHeatmapData({ tool: toolFilter }),
          getTopSessions({ tool: toolFilter, limit: 10 }),
          getDailyActivity({ tool: toolFilter, days: 30 }),
        ]);

      // Fetch hourly data if today is selected
      let hourlyUsage: DailyStat[] = [];
      if (isToday) {
        hourlyUsage = await getHourlyUsage({
          tool: toolFilter,
          date: todayDate,
        });
      }

      setData({
        stats,
        dailyUsage,
        hourlyUsage,
        dailyActivity,
        modelBreakdown,
        projectBreakdown,
        heatmap,
        topSessions,
        loading: false,
        error: null,
        isToday,
      });
    } catch (e) {
      setData((prev) => ({
        ...prev,
        loading: false,
        error: e instanceof Error ? e.message : String(e),
      }));
    }
  }, [toolFilter, dateRange?.start, dateRange?.end]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return { ...data, refresh: fetchData };
}

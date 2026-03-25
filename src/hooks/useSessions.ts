import { useState, useEffect, useCallback } from "react";
import { getSessions, getSessionDetail } from "../lib/tauri";
import type { SessionFilters, SessionPage, SessionDetail } from "../lib/types";

export function useSessions(initialFilters?: Partial<SessionFilters>) {
  const [filters, setFilters] = useState<SessionFilters>({
    page: 1,
    page_size: 50,
    sort_by: "start_time",
    sort_dir: "desc",
    ...initialFilters,
  });
  const [page, setPage] = useState<SessionPage | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchSessions = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await getSessions(filters);
      setPage(result);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [filters]);

  useEffect(() => {
    fetchSessions();
  }, [fetchSessions]);

  const updateFilters = useCallback(
    (updates: Partial<SessionFilters>) => {
      setFilters((prev) => ({ ...prev, ...updates, page: 1 }));
    },
    []
  );

  const setSort = useCallback((column: string) => {
    setFilters((prev) => ({
      ...prev,
      sort_by: column,
      sort_dir:
        prev.sort_by === column && prev.sort_dir === "desc" ? "asc" : "desc",
    }));
  }, []);

  const goToPage = useCallback((p: number) => {
    setFilters((prev) => ({ ...prev, page: p }));
  }, []);

  return {
    sessions: page?.sessions ?? [],
    totalCount: page?.total_count ?? 0,
    currentPage: page?.page ?? 1,
    pageSize: page?.page_size ?? 50,
    filters,
    loading,
    error,
    updateFilters,
    setSort,
    goToPage,
    refresh: fetchSessions,
  };
}

export function useSessionDetail(sessionId: string | null) {
  const [detail, setDetail] = useState<SessionDetail | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!sessionId) {
      setDetail(null);
      return;
    }
    setLoading(true);
    getSessionDetail(sessionId)
      .then(setDetail)
      .catch(() => setDetail(null))
      .finally(() => setLoading(false));
  }, [sessionId]);

  return { detail, loading };
}

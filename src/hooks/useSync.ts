import { useState, useCallback, useEffect, useRef } from "react";
import {
  syncData,
  getSyncStatus,
  onSyncProgress,
  onSyncComplete,
} from "../lib/tauri";
import { useSources } from "../contexts/SourceContext";
import type { SyncResult, SyncState } from "../lib/types";
import type { SyncProgress } from "../lib/tauri";

export type SyncStatusType = "idle" | "syncing" | "error";

export function useSync() {
  const [status, setStatus] = useState<SyncStatusType>("idle");
  const [progress, setProgress] = useState<SyncProgress | null>(null);
  const [lastResult, setLastResult] = useState<SyncResult | null>(null);
  const [syncStates, setSyncStates] = useState<SyncState[]>([]);
  const [error, setError] = useState<string | null>(null);
  const refreshCallbackRef = useRef<(() => void) | null>(null);
  const { refreshSources } = useSources();
  const refreshSourcesRef = useRef(refreshSources);
  refreshSourcesRef.current = refreshSources;

  // Set up event listeners once
  useEffect(() => {
    const unlistenProgress = onSyncProgress((p) => {
      setProgress(p);
      if (p.phase === "done") {
        setStatus("idle");
        setProgress(null);
        // Refresh sync states and sources
        getSyncStatus().then(setSyncStates).catch(() => {});
        refreshSourcesRef.current();
        // Notify any refresh callback
        refreshCallbackRef.current?.();
      }
    });

    const unlistenComplete = onSyncComplete((result) => {
      setLastResult(result);
      if (result.errors.length > 0) {
        setError(result.errors.join("; "));
      }
      setStatus("idle");
      // Refresh sources after sync
      refreshSourcesRef.current();
      // Notify any refresh callback
      refreshCallbackRef.current?.();
    });

    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, []);

  const triggerSync = useCallback(
    async (forceFull = false) => {
      if (status === "syncing") return;
      setStatus("syncing");
      setError(null);
      setProgress(null);
      try {
        await syncData(forceFull); // Returns immediately — work happens in background
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setError(msg);
        setStatus("error");
      }
    },
    [status]
  );

  const loadSyncStatus = useCallback(async () => {
    try {
      const states = await getSyncStatus();
      setSyncStates(states);
    } catch {
      // Ignore — may not have synced yet
    }
  }, []);

  // Load sync status on mount
  useEffect(() => {
    loadSyncStatus();
  }, [loadSyncStatus]);

  // Sync on window focus
  useEffect(() => {
    const handleFocus = () => {
      if (status !== "syncing") {
        triggerSync(false);
      }
    };
    window.addEventListener("focus", handleFocus);
    return () => window.removeEventListener("focus", handleFocus);
  }, [status, triggerSync]);

  /** Register a callback to be called when sync completes (for refreshing dashboard data) */
  const onComplete = useCallback((callback: () => void) => {
    refreshCallbackRef.current = callback;
  }, []);

  return {
    status,
    progress,
    lastResult,
    syncStates,
    error,
    triggerSync,
    onComplete,
  };
}

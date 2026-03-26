import { formatTokens } from "../../lib/format";
import type { DashboardStats } from "../../lib/types";
import type { SyncStatusType } from "../../hooks/useSync";
import type { SyncProgress } from "../../lib/tauri";
import { ToolIcon } from "../shared/ToolIcon";

interface Props {
  title: string;
  toolId?: string;
  stats: DashboardStats | null;
  syncStatus: SyncStatusType;
  syncProgress: SyncProgress | null;
  onRefresh: () => void;
}

export function TopBar({
  title,
  toolId,
  stats,
  syncStatus,
  syncProgress,
  onRefresh,
}: Props) {
  return (
    <div className="flex items-center justify-between mb-8 mt-4">
      <h2 className="font-serif text-[42px] text-text-primary flex items-center gap-3">
        {toolId && <ToolIcon tool={toolId} size={38} />}
        {title}
      </h2>
      <div className="flex items-center gap-4">
        {/* Sync status badge */}
        {syncStatus === "syncing" && syncProgress && (
          <div className="flex items-center gap-2 px-3 py-1 bg-white rounded-(--radius-badge) border border-border">
            <div className="w-3 h-3 border-2 border-text-secondary border-t-interactive rounded-full animate-spin" />
            <span className="text-xs text-text-secondary">
              {syncProgress.message}
            </span>
          </div>
        )}

        {stats && syncStatus !== "syncing" && (
          <div className="flex items-center gap-3 text-xs text-text-secondary">
            <span>
              <strong className="text-text-primary">{stats.streak}</strong> day
              streak
            </span>
            <span className="text-border">|</span>
            <span>
              <strong className="text-text-primary">
                {formatTokens(stats.tokens_today)}
              </strong>{" "}
              tokens today
            </span>
            <span className="text-border">|</span>
            <span>
              <strong className="text-text-primary">
                {stats.sessions_today}
              </strong>{" "}
              sessions
            </span>
          </div>
        )}

        <button
          onClick={onRefresh}
          disabled={syncStatus === "syncing"}
          className="p-2 rounded-(--radius-button) hover:bg-white border border-border transition-all duration-300 disabled:opacity-50"
          title="Refresh data"
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
            className={syncStatus === "syncing" ? "animate-spin" : ""}
          >
            <path
              d="M13.65 2.35A8 8 0 1 0 16 8h-2a6 6 0 1 1-1.76-4.24L10 6h6V0l-2.35 2.35z"
              fill="#8D8D83"
              transform="scale(0.85) translate(1, 1)"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}

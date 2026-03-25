import { useState } from "react";
import { useSessions, useSessionDetail } from "../hooks/useSessions";
import { useSync } from "../hooks/useSync";
import { TopBar } from "../components/layout/TopBar";
import { Badge } from "../components/shared/Badge";
import { ToolIcon } from "../components/shared/ToolIcon";
import { EmptyState } from "../components/shared/EmptyState";
import { formatTokens, formatCost, formatRelativeTime } from "../lib/format";
import type { Session } from "../lib/types";

export default function Sessions() {
  const sync = useSync();
  const {
    sessions,
    totalCount,
    currentPage,
    pageSize,
    filters,
    updateFilters,
    setSort,
    goToPage,
  } = useSessions();

  const [expandedId, setExpandedId] = useState<string | null>(null);
  const { detail } = useSessionDetail(expandedId);

  const totalPages = Math.ceil(totalCount / pageSize);

  const columns = [
    { key: "start_time", label: "Time" },
    { key: "tool", label: "Tool" },
    { key: "model", label: "Model" },
    { key: "project_name", label: "Project" },
    { key: "total_input_tokens", label: "Input" },
    { key: "total_output_tokens", label: "Output" },
    { key: "total_tokens", label: "Total" },
    { key: "estimated_cost", label: "Cost" },
  ];

  return (
    <div>
      <TopBar
        title="Sessions"
        stats={null}
        syncStatus={sync.status}
        syncProgress={sync.progress}
        onRefresh={() => sync.triggerSync(false)}
      />

      {/* Filters */}
      <div className="flex gap-2 mb-4 flex-wrap">
        <input
          type="text"
          placeholder="Search sessions..."
          value={filters.search ?? ""}
          onChange={(e) =>
            updateFilters({ search: e.target.value || undefined })
          }
          className="px-3 py-1.5 rounded-(--radius-button) border border-border bg-white text-sm text-text-primary placeholder:text-text-secondary focus:outline-none focus:border-interactive"
        />
        <select
          value={filters.tool ?? ""}
          onChange={(e) =>
            updateFilters({ tool: e.target.value || undefined })
          }
          className="px-3 py-1.5 rounded-(--radius-button) border border-border bg-white text-sm text-text-primary"
        >
          <option value="">All Tools</option>
          <option value="claude">Claude Code</option>
          <option value="codex">Codex</option>
        </select>
        <select
          value={filters.source ?? ""}
          onChange={(e) =>
            updateFilters({ source: e.target.value || undefined })
          }
          className="px-3 py-1.5 rounded-(--radius-button) border border-border bg-white text-sm text-text-primary"
        >
          <option value="">All Sources</option>
          <option value="cli">CLI</option>
          <option value="vscode">VS Code</option>
          <option value="subagent">Subagent</option>
        </select>
      </div>

      {/* Table */}
      <div className="bg-white rounded-(--radius-card) border border-border overflow-hidden">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-border">
              {columns.map((col) => (
                <th
                  key={col.key}
                  onClick={() => setSort(col.key)}
                  className="text-left px-4 py-3 text-xs text-text-secondary font-medium cursor-pointer hover:text-text-primary select-none"
                >
                  {col.label}
                  {filters.sort_by === col.key && (
                    <span className="ml-1">
                      {filters.sort_dir === "asc" ? "↑" : "↓"}
                    </span>
                  )}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {sessions.length === 0 ? (
              <tr>
                <td colSpan={columns.length}>
                  <EmptyState message="No sessions found" />
                </td>
              </tr>
            ) : (
              sessions.map((session) => (
                <SessionRow
                  key={session.id}
                  session={session}
                  expanded={expandedId === session.id}
                  onToggle={() =>
                    setExpandedId(
                      expandedId === session.id ? null : session.id
                    )
                  }
                  detail={expandedId === session.id ? detail : null}
                  colCount={columns.length}
                />
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="flex items-center justify-between mt-4 text-sm text-text-secondary">
          <span>
            {totalCount} sessions · Page {currentPage} of {totalPages}
          </span>
          <div className="flex gap-2">
            <button
              onClick={() => goToPage(currentPage - 1)}
              disabled={currentPage <= 1}
              className="px-3 py-1 rounded-(--radius-button) border border-border bg-white hover:bg-cream disabled:opacity-40"
            >
              Prev
            </button>
            <button
              onClick={() => goToPage(currentPage + 1)}
              disabled={currentPage >= totalPages}
              className="px-3 py-1 rounded-(--radius-button) border border-border bg-white hover:bg-cream disabled:opacity-40"
            >
              Next
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

function SessionRow({
  session,
  expanded,
  onToggle,
  detail,
  colCount,
}: {
  session: Session;
  expanded: boolean;
  onToggle: () => void;
  detail: import("../lib/types").SessionDetail | null;
  colCount: number;
}) {
  return (
    <>
      <tr
        onClick={onToggle}
        className="border-b border-border hover:bg-cream/50 cursor-pointer transition-colors duration-300"
      >
        <td className="px-4 py-3 text-text-secondary">
          {formatRelativeTime(session.start_time)}
        </td>
        <td className="px-4 py-3">
          <ToolIcon tool={session.tool} />
        </td>
        <td className="px-4 py-3">
          <Badge
            tool={session.tool as "claude" | "codex"}
            label={session.model ?? "—"}
          />
        </td>
        <td className="px-4 py-3 text-text-primary">
          {session.project_name ?? "—"}
        </td>
        <td className="px-4 py-3 text-text-secondary">
          {formatTokens(session.total_input_tokens)}
        </td>
        <td className="px-4 py-3 text-text-secondary">
          {formatTokens(session.total_output_tokens)}
        </td>
        <td className="px-4 py-3 font-medium text-text-primary">
          {formatTokens(session.total_tokens)}
        </td>
        <td className="px-4 py-3 text-text-secondary">
          {session.estimated_cost > 0 ? formatCost(session.estimated_cost) : "—"}
        </td>
      </tr>
      {expanded && detail && (
        <tr>
          <td colSpan={colCount} className="bg-cream/30 px-8 py-4">
            <p className="text-xs text-text-secondary mb-2">
              {detail.requests.length} requests
              {detail.children.length > 0 &&
                ` · ${detail.children.length} subagent threads`}
            </p>
            <div className="space-y-1">
              {detail.requests.slice(0, 20).map((req) => (
                <div
                  key={req.id}
                  className="flex items-center gap-4 text-xs text-text-secondary"
                >
                  <span className="w-20">
                    {new Date(req.timestamp).toLocaleTimeString([], {
                      hour: "2-digit",
                      minute: "2-digit",
                    })}
                  </span>
                  <span>{req.model ?? "—"}</span>
                  <span>
                    {formatTokens(req.input_tokens)} in /{" "}
                    {formatTokens(req.output_tokens)} out
                  </span>
                  <span className="font-medium text-text-primary">
                    {formatTokens(req.total_tokens)}
                  </span>
                </div>
              ))}
              {detail.requests.length > 20 && (
                <p className="text-xs text-text-secondary">
                  ...and {detail.requests.length - 20} more
                </p>
              )}
            </div>
          </td>
        </tr>
      )}
    </>
  );
}

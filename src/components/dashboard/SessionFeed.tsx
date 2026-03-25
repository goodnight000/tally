import { useEffect, useState } from "react";
import { getSessions } from "../../lib/tauri";
import { ToolIcon } from "../shared/ToolIcon";
import { Badge } from "../shared/Badge";
import { EmptyState } from "../shared/EmptyState";
import { formatTokens, formatCost, formatRelativeTime } from "../../lib/format";
import type { Session } from "../../lib/types";

interface Props {
  toolFilter?: string;
  selectedDate?: string | null;
}

export function SessionFeed({ toolFilter, selectedDate }: Props) {
  const [sessions, setSessions] = useState<Session[]>([]);
  useEffect(() => {
    getSessions({
      tool: toolFilter,
      start_date: selectedDate ? selectedDate + "T00:00:00Z" : undefined,
      end_date: selectedDate ? selectedDate + "T23:59:59Z" : undefined,
      page: 1,
      page_size: 20,
      sort_by: "start_time",
      sort_dir: "desc",
    })
      .then((page) => setSessions(page.sessions.filter((s) => s.source !== "subagent")));
  }, [toolFilter, selectedDate]);

  return (
    <div className="bg-white rounded-(--radius-card) p-(--spacing-card-padding) border border-border">
      <h3 className="font-serif text-xl font-semibold text-text-primary mb-4">
        Recent Sessions
        {selectedDate && (
          <span className="text-sm font-sans font-normal text-text-secondary ml-2">
            {selectedDate}
          </span>
        )}
      </h3>
      {sessions.length === 0 ? (
        <EmptyState message="No sessions recorded yet. Usage will appear here automatically." />
      ) : (
        <div className="space-y-2">
          {sessions.map((session) => (
            <div
              key={session.id}
              className="flex items-center gap-3 py-2 border-b border-border last:border-0"
            >
              <span className="text-xs text-text-secondary w-16 shrink-0">
                {formatRelativeTime(session.start_time)}
              </span>
              <ToolIcon tool={session.tool} />
              <Badge
                tool={session.tool as "claude" | "codex"}
                label={session.model ?? "—"}
              />
              <span className="text-sm text-text-primary font-medium flex-1 truncate">
                {session.project_name ?? session.title ?? "—"}
              </span>
              <span className="text-sm text-text-secondary tabular-nums">
                {formatTokens(session.total_tokens)}
              </span>
              {session.estimated_cost > 0 && (
                <span className="text-xs text-text-secondary tabular-nums">
                  {formatCost(session.estimated_cost)}
                </span>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

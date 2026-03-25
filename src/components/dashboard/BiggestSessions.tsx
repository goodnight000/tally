import { ToolIcon } from "../shared/ToolIcon";
import { EmptyState } from "../shared/EmptyState";
import { formatTokens, formatCost, formatRelativeTime } from "../../lib/format";
import type { Session } from "../../lib/types";

interface Props {
  sessions: Session[];
}

export function BiggestSessions({ sessions }: Props) {
  if (sessions.length === 0) {
    return (
      <div className="bg-white rounded-(--radius-card) p-(--spacing-card-padding) border border-border">
        <h3 className="font-serif text-xl font-semibold text-text-primary mb-4">
          Biggest Sessions
        </h3>
        <EmptyState message="No sessions recorded" />
      </div>
    );
  }

  return (
    <div className="bg-white rounded-(--radius-card) p-(--spacing-card-padding) border border-border">
      <h3 className="font-serif text-xl font-semibold text-text-primary mb-4">
        Biggest Sessions
      </h3>
      <div className="space-y-2">
        {sessions.map((session, i) => (
          <div
            key={session.id}
            className="flex items-center gap-3 py-1.5"
          >
            <span className="text-xs text-text-secondary w-5 text-right">
              {i + 1}
            </span>
            <ToolIcon tool={session.tool} />
            <span className="text-sm text-text-primary flex-1 truncate">
              {session.project_name ?? session.title ?? "—"}
            </span>
            <span className="text-xs text-text-secondary">
              {formatRelativeTime(session.start_time)}
            </span>
            <span className="text-sm font-medium text-text-primary tabular-nums">
              {formatTokens(session.total_tokens)}
            </span>
            {session.estimated_cost > 0 && (
              <span className="text-xs text-text-secondary tabular-nums w-16 text-right">
                {formatCost(session.estimated_cost)}
              </span>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

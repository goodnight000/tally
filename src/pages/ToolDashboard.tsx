import { useEffect, useState } from "react";
import { useDashboard } from "../hooks/useDashboard";
import { useSync } from "../hooks/useSync";
import { TopBar } from "../components/layout/TopBar";
import { DateRangePicker } from "../components/shared/DateRangePicker";
import { DashboardGrid } from "./Home";
import { daysAgo, today } from "../lib/format";

const DEFAULT_RANGE = { start: daysAgo(30), end: today() + "T23:59:59Z" };

interface Props {
  toolId: string;
  displayName: string;
}

export default function ToolDashboard({ toolId, displayName }: Props) {
  const sync = useSync();
  const [dateRange, setDateRange] = useState<{
    start: string;
    end: string;
  } | null>(DEFAULT_RANGE);
  const dashboard = useDashboard(toolId, dateRange);

  useEffect(() => {
    sync.onComplete(() => dashboard.refresh());
  }, [sync.onComplete, dashboard.refresh]);

  return (
    <div>
      <TopBar
        title={displayName}
        toolId={toolId}
        stats={dashboard.stats}
        syncStatus={sync.status}
        syncProgress={sync.progress}
        onRefresh={() => sync.triggerSync(false)}
      />
      <div className="flex justify-end mb-4">
        <DateRangePicker onChange={setDateRange} />
      </div>
      <DashboardGrid
        dashboard={dashboard}
        toolFilter={toolId}
        isToday={dashboard.isToday}
        monthAnchor={dateRange?.end}
      />
    </div>
  );
}

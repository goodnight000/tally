import { useState, useEffect } from "react";
import { useDashboard } from "../hooks/useDashboard";
import { useSync } from "../hooks/useSync";
import { TopBar } from "../components/layout/TopBar";
import { DateRangePicker } from "../components/shared/DateRangePicker";
import { DailyUsageChart } from "../components/charts/DailyUsageChart";
import { StatCards } from "../components/dashboard/StatCards";
import { ModelBreakdown } from "../components/dashboard/ModelBreakdown";
import { Heatmap } from "../components/dashboard/Heatmap";
import { SessionFeed } from "../components/dashboard/SessionFeed";
import { TopProjects } from "../components/dashboard/TopProjects";
import { BiggestSessions } from "../components/dashboard/BiggestSessions";
import { daysAgo, today } from "../lib/format";

const DEFAULT_RANGE = { start: daysAgo(30), end: today() + "T23:59:59Z" };

export default function Home() {
  const sync = useSync();
  const [dateRange, setDateRange] = useState<{
    start: string;
    end: string;
  } | null>(DEFAULT_RANGE);
  const dashboard = useDashboard(undefined, dateRange);

  useEffect(() => {
    sync.onComplete(() => dashboard.refresh());
  }, [sync.onComplete, dashboard.refresh]);

  return (
    <div>
      <TopBar
        title="Welcome back"
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
        toolFilter={undefined}
        isToday={dashboard.isToday}
        monthAnchor={dateRange?.end}
      />
    </div>
  );
}

export function DashboardGrid({
  dashboard,
  toolFilter,
  isToday,
  monthAnchor,
}: {
  dashboard: ReturnType<typeof useDashboard>;
  toolFilter?: string;
  isToday?: boolean;
  monthAnchor?: string;
}) {
  const [selectedDate, setSelectedDate] = useState<string | null>(null);

  return (
    <div className="space-y-(--spacing-card-gap)">
      <StatCards stats={dashboard.stats} />
      <DailyUsageChart
        data={isToday ? dashboard.hourlyUsage : dashboard.dailyUsage}
        onDayClick={setSelectedDate}
        isHourly={isToday}
      />
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-(--spacing-card-gap)">
        <ModelBreakdown data={dashboard.modelBreakdown} />
        <Heatmap
          toolFilter={toolFilter}
          selectedDate={selectedDate}
          monthAnchor={monthAnchor}
          onDateSelect={setSelectedDate}
        />
      </div>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-(--spacing-card-gap)">
        <SessionFeed toolFilter={toolFilter} selectedDate={selectedDate} />
        <TopProjects data={dashboard.projectBreakdown} toolFilter={toolFilter} />
      </div>
      <BiggestSessions sessions={dashboard.topSessions} />
    </div>
  );
}

import { useMemo, useState, useRef } from "react";
import { getToolColor } from "../../lib/constants";
import { formatTokens, formatCost } from "../../lib/format";
import { EmptyState } from "../shared/EmptyState";
import type { DailyActivity, DailyStat } from "../../lib/types";

interface Props {
  data: DailyActivity[];
  dailyUsage?: DailyStat[];
  toolFilter?: string;
}

function hexToRgb(hex: string): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `${r}, ${g}, ${b}`;
}

const DAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

interface DayData {
  name: string;
  count: number;
  tokens: number;
  cost: number;
  activeDays: number;
  intensity: number;
}

export function Heatmap({ data, dailyUsage, toolFilter }: Props) {
  const color = getToolColor(toolFilter);
  const rgb = hexToRgb(color);
  const [hoveredDay, setHoveredDay] = useState<number | null>(null);
  const [tooltipPos, setTooltipPos] = useState({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  const grid = useMemo(() => {
    const dayCounts = new Array(7).fill(0);
    const dayActiveDays = new Array(7).fill(0);
    const dayTokens = new Array(7).fill(0);
    const dayCost = new Array(7).fill(0);

    for (const entry of data) {
      const d = new Date(entry.date + "T12:00:00");
      const dow = d.getDay();
      dayCounts[dow] += entry.count;
      dayActiveDays[dow]++;
    }

    if (dailyUsage) {
      for (const entry of dailyUsage) {
        const d = new Date(entry.date + "T12:00:00");
        const dow = d.getDay();
        dayTokens[dow] += entry.total_tokens;
        dayCost[dow] += entry.estimated_cost;
      }
    }

    const maxCount = Math.max(1, ...dayCounts);
    return DAYS.map((name, i): DayData => ({
      name,
      count: dayCounts[i],
      tokens: dayTokens[i],
      cost: dayCost[i],
      activeDays: dayActiveDays[i],
      intensity: dayCounts[i] / maxCount,
    }));
  }, [data, dailyUsage]);

  const handleMouseMove = (e: React.MouseEvent, dayIndex: number) => {
    if (!containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    setTooltipPos({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    });
    setHoveredDay(dayIndex);
  };

  return (
    <div className="bg-white rounded-(--radius-card) p-(--spacing-card-padding) border border-border">
      <h3 className="font-serif text-xl font-semibold text-text-primary mb-4">
        Activity by Day
      </h3>
      {data.length === 0 ? (
        <EmptyState message="Start using Claude Code or Codex to see patterns" />
      ) : (
        <div className="relative" ref={containerRef}>
          <div className="flex gap-2">
            {grid.map((day, i) => (
              <div
                key={day.name}
                className="flex-1 text-center"
                onMouseMove={(e) => handleMouseMove(e, i)}
                onMouseLeave={() => setHoveredDay(null)}
              >
                <div
                  className="w-full aspect-square rounded-lg mb-1 transition-all duration-300 cursor-default"
                  style={{
                    backgroundColor:
                      day.count === 0
                        ? "#F0F0E4"
                        : `rgba(${rgb}, ${0.15 + day.intensity * 0.75})`,
                  }}
                />
                <span className="text-[10px] text-text-secondary">{day.name}</span>
              </div>
            ))}
          </div>

          {/* Tooltip — follows cursor smoothly like Recharts */}
          <div
            className="absolute pointer-events-none z-10"
            style={{
              left: tooltipPos.x,
              top: tooltipPos.y - 12,
              transform: "translate(-50%, -100%)",
              opacity: hoveredDay !== null && grid[hoveredDay]?.count > 0 ? 1 : 0,
              transition: "left 100ms ease-out, top 100ms ease-out, opacity 150ms ease",
            }}
          >
            <div
              className="bg-white px-3 py-2"
              style={{
                borderRadius: 8,
                border: "1px solid #E8E8E0",
                fontSize: 12,
                whiteSpace: "nowrap",
              }}
            >
              {hoveredDay !== null && (
                <>
                  <p className="text-text-primary font-medium">
                    {grid[hoveredDay].name}
                  </p>
                  <p className="text-text-secondary">
                    {grid[hoveredDay].count} sessions
                  </p>
                  {grid[hoveredDay].tokens > 0 && (
                    <p className="text-text-secondary">
                      {formatTokens(grid[hoveredDay].tokens)} tokens
                    </p>
                  )}
                  {grid[hoveredDay].cost > 0 && (
                    <p className="text-text-secondary">
                      {formatCost(grid[hoveredDay].cost)}
                    </p>
                  )}
                </>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

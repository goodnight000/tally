import { useEffect, useMemo, useRef, useState } from "react";
import { getToolColor } from "../../lib/constants";
import { getDailyUsage } from "../../lib/tauri";
import {
  formatCost,
  formatMonthLabel,
  formatTokens,
  getCalendarBounds,
  getDateRange,
  getMonthBounds,
  getMonthKey,
  isSameDay,
  shiftMonth,
  today,
} from "../../lib/format";
import type { DailyStat } from "../../lib/types";

interface Props {
  toolFilter?: string;
  selectedDate?: string | null;
  monthAnchor?: string;
  onDateSelect?: (date: string | null) => void;
}

interface CalendarCell {
  date: string;
  dayNumber: number;
  inMonth: boolean;
  isToday: boolean;
  isSelected: boolean;
  isFuture: boolean;
  totalTokens: number;
  sessionCount: number;
  estimatedCost: number;
  intensity: number;
}

function hexToRgb(hex: string): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `${r}, ${g}, ${b}`;
}

function formatCalendarDate(date: string): string {
  return new Date(`${date}T12:00:00`).toLocaleDateString("en-US", {
    weekday: "short",
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

const WEEKDAY_LABELS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

export function Heatmap({
  toolFilter,
  selectedDate,
  monthAnchor,
  onDateSelect,
}: Props) {
  const color = getToolColor(toolFilter);
  const rgb = hexToRgb(color);
  const anchorDate = selectedDate ?? monthAnchor ?? today();
  const [visibleMonth, setVisibleMonth] = useState(getMonthKey(anchorDate));
  const [monthUsage, setMonthUsage] = useState<DailyStat[]>([]);
  const [loading, setLoading] = useState(true);
  const [hoveredDate, setHoveredDate] = useState<string | null>(null);
  const [tooltipPos, setTooltipPos] = useState({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);
  const todayDate = today();

  useEffect(() => {
    setVisibleMonth(getMonthKey(anchorDate));
  }, [anchorDate]);

  useEffect(() => {
    let cancelled = false;
    const monthBounds = getMonthBounds(visibleMonth);

    setLoading(true);
    getDailyUsage({
      tool: toolFilter,
      start: monthBounds.start,
      end: `${monthBounds.end}T23:59:59Z`,
    })
      .then((result) => {
        if (!cancelled) {
          setMonthUsage(result);
          setLoading(false);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setMonthUsage([]);
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [toolFilter, visibleMonth]);

  const usageByDate = useMemo(() => {
    const map = new Map<string, DailyStat>();
    for (const entry of monthUsage) {
      map.set(entry.date, entry);
    }
    return map;
  }, [monthUsage]);

  const monthSummary = useMemo(() => {
    let totalTokens = 0;
    let totalCost = 0;

    for (const entry of monthUsage) {
      totalTokens += entry.total_tokens;
      totalCost += entry.estimated_cost;
    }

    return {
      totalTokens,
      totalCost,
    };
  }, [monthUsage]);

  const cells = useMemo(() => {
    const monthMaxTokens = Math.max(0, ...monthUsage.map((entry) => entry.total_tokens));
    const calendarBounds = getCalendarBounds(visibleMonth);

    return getDateRange(calendarBounds.start, calendarBounds.end).map((date): CalendarCell => {
      const stat = usageByDate.get(date);
      const totalTokens = stat?.total_tokens ?? 0;
      const isFuture = date > todayDate;

      return {
        date,
        dayNumber: Number(date.slice(8, 10)),
        inMonth: date.startsWith(visibleMonth),
        isToday: isSameDay(date, todayDate),
        isSelected: isSameDay(date, selectedDate),
        isFuture,
        totalTokens,
        sessionCount: stat?.session_count ?? 0,
        estimatedCost: stat?.estimated_cost ?? 0,
        intensity: totalTokens > 0 && monthMaxTokens > 0 ? totalTokens / monthMaxTokens : 0,
      };
    });
  }, [monthUsage, selectedDate, todayDate, usageByDate, visibleMonth]);

  const hoveredCell = hoveredDate
    ? cells.find((cell) => cell.date === hoveredDate) ?? null
    : null;

  const handleMouseMove = (e: React.MouseEvent, date: string) => {
    if (!containerRef.current) return;
    const rect = containerRef.current.getBoundingClientRect();
    setTooltipPos({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    });
    setHoveredDate(date);
  };

  const getCellBackground = (cell: CalendarCell): string => {
    if (!cell.inMonth) return "rgba(240, 240, 228, 0.45)";
    if (cell.isFuture) return "rgba(240, 240, 228, 0.7)";
    if (cell.totalTokens === 0) return "#F0F0E4";
    return `rgba(${rgb}, ${0.18 + cell.intensity * 0.72})`;
  };

  return (
    <div className="bg-white rounded-(--radius-card) p-(--spacing-card-padding) border border-border">
      <div className="mb-4">
        <h3 className="font-serif text-xl font-semibold text-text-primary mb-4">
          Activity by Day
        </h3>
        <div className="flex items-center justify-center gap-3">
          <button
            type="button"
            className="h-8 w-8 rounded-full border border-border text-text-secondary hover:text-text-primary transition-colors"
            onClick={() => setVisibleMonth((current) => shiftMonth(current, -1))}
            aria-label="Previous month"
          >
            ←
          </button>
          <div className="min-w-40 text-center text-base font-semibold text-text-primary">
            {formatMonthLabel(visibleMonth)}
          </div>
          <button
            type="button"
            className="h-8 w-8 rounded-full border border-border text-text-secondary hover:text-text-primary transition-colors"
            onClick={() => setVisibleMonth((current) => shiftMonth(current, 1))}
            aria-label="Next month"
          >
            →
          </button>
        </div>
      </div>

      <div className="relative" ref={containerRef}>
        <div className="grid grid-cols-7 gap-2 mb-2">
          {WEEKDAY_LABELS.map((day) => (
            <div
              key={day}
              className="text-[11px] font-medium uppercase tracking-[0.12em] text-text-secondary text-center"
            >
              {day}
            </div>
          ))}
        </div>

        <div className="grid grid-cols-7 gap-2">
          {cells.map((cell) => {
            const isInteractive = cell.inMonth && !cell.isFuture;

            return (
              <button
                key={cell.date}
                type="button"
                disabled={!isInteractive}
                className="aspect-square rounded-xl p-2 text-left transition-all duration-200 disabled:cursor-default"
                style={{
                  backgroundColor: getCellBackground(cell),
                  border: cell.isSelected
                    ? `2px solid ${color}`
                    : cell.isToday
                      ? `1px solid rgba(${rgb}, 0.7)`
                      : "1px solid transparent",
                  opacity: cell.inMonth ? 1 : 0.55,
                }}
                onMouseMove={(e) => handleMouseMove(e, cell.date)}
                onMouseLeave={() => setHoveredDate(null)}
                onClick={() =>
                  onDateSelect?.(cell.isSelected ? null : cell.date)
                }
                aria-label={`${formatCalendarDate(cell.date)}: ${formatTokens(cell.totalTokens)} tokens`}
              >
                <div className="flex h-full flex-col justify-between">
                  <span
                    className="text-xs font-medium"
                    style={{
                      color: cell.inMonth ? "#1A1A1A" : "#8D8D83",
                    }}
                  >
                    {cell.dayNumber}
                  </span>
                  {cell.inMonth && cell.totalTokens > 0 && (
                    <span className="text-[10px] text-text-primary/80 tabular-nums">
                      {formatTokens(cell.totalTokens)}
                    </span>
                  )}
                </div>
              </button>
            );
          })}
        </div>

        <div
          className="absolute pointer-events-none z-10"
          style={{
            left: tooltipPos.x,
            top: tooltipPos.y - 12,
            transform: "translate(-50%, -100%)",
            opacity: hoveredCell && hoveredCell.inMonth && !hoveredCell.isFuture ? 1 : 0,
            transition: "left 100ms ease-out, top 100ms ease-out, opacity 150ms ease",
          }}
        >
          {hoveredCell && hoveredCell.inMonth && !hoveredCell.isFuture && (
            <div
              className="bg-white px-3 py-2 shadow-sm"
              style={{
                borderRadius: 8,
                border: "1px solid #E8E8E0",
                fontSize: 12,
                whiteSpace: "nowrap",
              }}
            >
              <p className="text-text-primary font-medium">
                {formatCalendarDate(hoveredCell.date)}
              </p>
              <p className="text-text-secondary">
                {formatTokens(hoveredCell.totalTokens)} tokens
              </p>
              <p className="text-text-secondary">
                {hoveredCell.sessionCount} sessions
              </p>
              {hoveredCell.estimatedCost > 0 && (
                <p className="text-text-secondary">
                  {formatCost(hoveredCell.estimatedCost)}
                </p>
              )}
            </div>
          )}
        </div>
      </div>

      <div className="grid grid-cols-2 gap-3 mt-4 pt-4 border-t border-border">
        <div>
          <p className="text-[11px] uppercase tracking-[0.12em] text-text-secondary mb-1">
            Month Tokens
          </p>
          <p className="text-sm font-medium text-text-primary">
            {loading ? "Loading..." : formatTokens(monthSummary.totalTokens)}
          </p>
        </div>
        <div>
          <p className="text-[11px] uppercase tracking-[0.12em] text-text-secondary mb-1">
            Month Spending
          </p>
          <p className="text-sm font-medium text-text-primary">
            {loading ? "Loading..." : formatCost(monthSummary.totalCost)}
          </p>
        </div>
      </div>
    </div>
  );
}

import { useMemo, useState } from "react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";
import { TOOL_COLORS } from "../../lib/constants";
import { formatTokens, formatCost, formatDate } from "../../lib/format";
import { EmptyState } from "../shared/EmptyState";
import type { DailyStat } from "../../lib/types";

const TOOL_DISPLAY_NAMES: Record<string, string> = {
  claude: "Claude Code",
  codex: "Codex CLI",
  cline: "Cline",
  kilo: "Kilo Code",
  roo: "Roo Code",
  opencode: "OpenCode",
  openclaw: "OpenClaw",
};

interface Props {
  data: DailyStat[];
  onDayClick?: (date: string) => void;
  isHourly?: boolean;
}

type ViewMode = "tokens" | "cost";

export function DailyUsageChart({ data, onDayClick, isHourly }: Props) {
  const [hiddenTools, setHiddenTools] = useState<Set<string>>(new Set());
  const [viewMode, setViewMode] = useState<ViewMode>("tokens");

  // Discover which tools are present in the data
  const tools = useMemo(() => {
    const toolSet = new Set<string>();
    for (const d of data) {
      toolSet.add(d.tool);
    }
    return Array.from(toolSet).sort();
  }, [data]);

  const chartData = useMemo(() => {
    const byKey: Record<string, Record<string, number> & { key: string }> = {};

    for (const d of data) {
      const key = d.date;
      if (!byKey[key]) {
        const entry: Record<string, number> & { key: string } = { key } as any;
        for (const t of tools) {
          entry[t] = 0;
          entry[`${t}_cost`] = 0;
        }
        byKey[key] = entry;
      }
      byKey[key][d.tool] = (byKey[key][d.tool] || 0) + d.total_tokens;
      byKey[key][`${d.tool}_cost`] = (byKey[key][`${d.tool}_cost`] || 0) + d.estimated_cost;
    }

    const values = Object.values(byKey).sort((a, b) => a.key.localeCompare(b.key));

    if (isHourly) {
      const hourMap = new Map(values.map((v) => [v.key, v]));
      const full: typeof values = [];
      for (let h = 0; h < 24; h++) {
        const key = h.toString().padStart(2, "0");
        if (hourMap.has(key)) {
          full.push(hourMap.get(key)!);
        } else {
          const entry: Record<string, number> & { key: string } = { key } as any;
          for (const t of tools) {
            entry[t] = 0;
            entry[`${t}_cost`] = 0;
          }
          full.push(entry);
        }
      }
      return full;
    }

    return values;
  }, [data, isHourly, tools]);

  if (chartData.length === 0) {
    return (
      <div className="bg-white rounded-(--radius-card) p-(--spacing-card-padding) border border-border">
        <h3 className="font-serif text-xl font-semibold text-text-primary mb-4">
          {isHourly ? "Hourly Usage" : "Daily Usage"}
        </h3>
        <EmptyState message="No usage data yet" />
      </div>
    );
  }

  const toggleTool = (tool: string) => {
    setHiddenTools((prev) => {
      const next = new Set(prev);
      if (next.has(tool)) next.delete(tool);
      else next.add(tool);
      return next;
    });
  };

  const formatter = viewMode === "cost" ? formatCost : formatTokens;
  const yFormatter = viewMode === "cost"
    ? (v: number) => formatCost(v)
    : (v: number) => formatTokens(v);

  const xFormatter = isHourly
    ? (key: string) => `${key}:00`
    : (key: string) => formatDate(key);

  const getDisplayName = (toolId: string) =>
    TOOL_DISPLAY_NAMES[toolId] || toolId;

  return (
    <div className="bg-white rounded-(--radius-card) p-(--spacing-card-padding) border border-border">
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-serif text-xl font-semibold text-text-primary">
          {isHourly ? "Hourly Usage" : "Daily Usage"}
        </h3>
        <div className="flex gap-1">
          <button
            onClick={() => setViewMode("tokens")}
            className={`px-3 py-1 rounded-(--radius-button) text-xs transition-all duration-300 ${
              viewMode === "tokens"
                ? "bg-text-primary text-white"
                : "bg-white text-text-secondary hover:bg-cream border border-border"
            }`}
          >
            Tokens
          </button>
          <button
            onClick={() => setViewMode("cost")}
            className={`px-3 py-1 rounded-(--radius-button) text-xs transition-all duration-300 ${
              viewMode === "cost"
                ? "bg-text-primary text-white"
                : "bg-white text-text-secondary hover:bg-cream border border-border"
            }`}
          >
            Cost
          </button>
        </div>
      </div>
      <ResponsiveContainer width="100%" height={280}>
        <AreaChart
          data={chartData}
          onClick={(e) => {
            if (e?.activeLabel && onDayClick && !isHourly) {
              onDayClick(e.activeLabel as string);
            }
          }}
        >
          <CartesianGrid strokeDasharray="3 3" stroke="#E8E8E0" />
          <XAxis
            dataKey="key"
            tickFormatter={xFormatter}
            tick={{ fontSize: 11, fill: "#8D8D83" }}
            axisLine={{ stroke: "#E8E8E0" }}
            interval={isHourly ? 2 : undefined}
          />
          <YAxis
            tickFormatter={yFormatter}
            tick={{ fontSize: 11, fill: "#8D8D83" }}
            axisLine={{ stroke: "#E8E8E0" }}
            width={65}
          />
          <Tooltip
            formatter={(value: unknown, name: unknown) => {
              const toolId = (name as string).replace("_cost", "");
              return [formatter(value as number), getDisplayName(toolId)];
            }}
            labelFormatter={(label: unknown) =>
              isHourly ? `${label}:00` : formatDate(label as string)
            }
            contentStyle={{
              borderRadius: 8,
              border: "1px solid #E8E8E0",
              fontSize: 12,
            }}
          />
          <Legend
            onClick={(e) => {
              const key = e.dataKey as string;
              toggleTool(key.replace("_cost", ""));
            }}
            formatter={(value) => {
              const toolId = value.replace("_cost", "");
              return (
                <span
                  style={{
                    color: hiddenTools.has(toolId) ? "#8D8D83" : "#1A1A1A",
                    fontSize: 12,
                    cursor: "pointer",
                  }}
                >
                  {getDisplayName(toolId)}
                </span>
              );
            }}
          />
          {tools.map((tool) => {
            if (hiddenTools.has(tool)) return null;
            const dataKey = viewMode === "cost" ? `${tool}_cost` : tool;
            const color = TOOL_COLORS[tool] || TOOL_COLORS.aggregate;
            return (
              <Area
                key={tool}
                type="monotone"
                dataKey={dataKey}
                stackId="1"
                stroke={color}
                fill={color}
                fillOpacity={0.3}
                animationDuration={800}
              />
            );
          })}
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}

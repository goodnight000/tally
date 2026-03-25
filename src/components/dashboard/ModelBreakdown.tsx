import {
  BarChart,
  Bar,
  Cell,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { TOOL_COLORS } from "../../lib/constants";
import { formatTokens, formatCost } from "../../lib/format";
import { EmptyState } from "../shared/EmptyState";
import type { ModelBreakdown as ModelBreakdownType } from "../../lib/types";

interface Props {
  data: ModelBreakdownType[];
}

export function ModelBreakdown({ data }: Props) {
  if (data.length === 0) {
    return (
      <div className="bg-white rounded-(--radius-card) p-(--spacing-card-padding) border border-border">
        <h3 className="font-serif text-xl font-semibold text-text-primary mb-4">
          Model Breakdown
        </h3>
        <EmptyState message="No model data available" />
      </div>
    );
  }

  const chartData = data.map((d) => ({
    model: d.model,
    tokens: d.total_tokens,
    cost: d.estimated_cost,
    fill: TOOL_COLORS[d.tool] || TOOL_COLORS.aggregate,
  }));

  return (
    <div className="bg-white rounded-(--radius-card) p-(--spacing-card-padding) border border-border">
      <h3 className="font-serif text-xl font-semibold text-text-primary mb-4">
        Model Breakdown
      </h3>
      <ResponsiveContainer width="100%" height={Math.max(data.length * 40, 120)}>
        <BarChart data={chartData} layout="vertical" barSize={20}>
          <XAxis
            type="number"
            tickFormatter={formatTokens}
            tick={{ fontSize: 11, fill: "#8D8D83" }}
            axisLine={{ stroke: "#E8E8E0" }}
          />
          <YAxis
            type="category"
            dataKey="model"
            tick={{ fontSize: 12, fill: "#1A1A1A" }}
            axisLine={false}
            tickLine={false}
            width={120}
          />
          <Tooltip
            cursor={{ fill: "rgba(0,0,0,0.04)" }}
            content={({ active, payload }) => {
              if (!active || !payload?.[0]) return null;
              const d = payload[0].payload as { model: string; tokens: number; cost: number; fill: string };
              return (
                <div style={{ borderRadius: 8, border: "1px solid #E8E8E0", fontSize: 12, backgroundColor: "white", padding: "8px 12px" }}>
                  <p style={{ color: d.fill, fontWeight: 500, marginBottom: 2 }}>{d.model}</p>
                  <p style={{ color: "#8D8D83" }}>
                    {formatTokens(d.tokens)}
                    {d.cost > 0 ? ` · ${formatCost(d.cost)}` : ""}
                  </p>
                </div>
              );
            }}
          />
          <Bar dataKey="tokens" radius={[0, 4, 4, 0]} animationDuration={800}>
            {chartData.map((entry, i) => (
              <Cell key={i} fill={entry.fill} />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}

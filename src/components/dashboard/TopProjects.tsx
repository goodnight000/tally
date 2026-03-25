import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { formatTokens, formatCost } from "../../lib/format";
import { EmptyState } from "../shared/EmptyState";
import { getToolColor } from "../../lib/constants";
import type { ProjectSummary } from "../../lib/types";

interface Props {
  data: ProjectSummary[];
  toolFilter?: string;
}

export function TopProjects({ data, toolFilter }: Props) {
  if (data.length === 0) {
    return (
      <div className="bg-white rounded-(--radius-card) p-(--spacing-card-padding) border border-border">
        <h3 className="font-serif text-xl font-semibold text-text-primary mb-4">
          Top Projects
        </h3>
        <EmptyState message="No project data yet" />
      </div>
    );
  }

  const chartData = data.slice(0, 10).map((d) => ({
    name: d.project_name,
    tokens: d.total_tokens,
    sessions: d.session_count,
    cost: d.estimated_cost,
  }));

  return (
    <div className="bg-white rounded-(--radius-card) p-(--spacing-card-padding) border border-border">
      <h3 className="font-serif text-xl font-semibold text-text-primary mb-4">
        Top Projects
      </h3>
      <ResponsiveContainer width="100%" height={Math.max(chartData.length * 36, 100)}>
        <BarChart data={chartData} layout="vertical" barSize={16}>
          <XAxis
            type="number"
            tickFormatter={formatTokens}
            tick={{ fontSize: 11, fill: "#8D8D83" }}
            axisLine={{ stroke: "#E8E8E0" }}
          />
          <YAxis
            type="category"
            dataKey="name"
            tick={{ fontSize: 11, fill: "#1A1A1A" }}
            axisLine={false}
            tickLine={false}
            width={100}
          />
          <Tooltip
            formatter={(value: unknown, _name: unknown, props: unknown) => {
              const p = props as { payload?: { cost?: number } };
              const cost = p?.payload?.cost;
              return [
                `${formatTokens(value as number)}${cost && cost > 0 ? ` · ${formatCost(cost)}` : ""}`,
                "Tokens",
              ];
            }}
            contentStyle={{ borderRadius: 8, border: "1px solid #E8E8E0", fontSize: 12 }}
          />
          <Bar
            dataKey="tokens"
            fill={getToolColor(toolFilter)}
            radius={[0, 4, 4, 0]}
            animationDuration={800}
          />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}

import { StatCard } from "../shared/StatCard";
import { AnimatedNumber } from "../shared/AnimatedNumber";
import { formatTokens, formatCost } from "../../lib/format";
import type { DashboardStats } from "../../lib/types";

interface Props {
  stats: DashboardStats | null;
}

export function StatCards({ stats }: Props) {
  if (!stats) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-3 gap-(--spacing-card-gap)">
        <StatCard label="Input vs Output"><span className="text-text-secondary">—</span></StatCard>
        <StatCard label="Total Tokens"><span className="text-text-secondary">—</span></StatCard>
        <StatCard label="Estimated Cost"><span className="text-text-secondary">—</span></StatCard>
      </div>
    );
  }

  const totalInput = stats.total_input_tokens;
  const totalOutput = stats.total_output_tokens;
  const ioRatio = totalInput > 0 ? (totalOutput / totalInput).toFixed(2) : "—";

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-(--spacing-card-gap)">
      <StatCard label="Input vs Output" subtitle={`${ioRatio}x output/input ratio`}>
        <div className="flex items-end gap-3">
          <AnimatedNumber value={totalInput} formatter={formatTokens} />
          <span className="text-lg text-text-secondary font-sans not-italic">/</span>
          <AnimatedNumber value={totalOutput} formatter={formatTokens} />
        </div>
      </StatCard>

      <StatCard label="Total Tokens">
        <AnimatedNumber value={stats.total_tokens} formatter={formatTokens} />
      </StatCard>

      <StatCard label="Estimated Cost" subtitle="at API rates">
        <AnimatedNumber
          value={Math.round(stats.estimated_cost * 100)}
          formatter={(n) => formatCost(n / 100)}
        />
      </StatCard>
    </div>
  );
}

import { useState, useEffect } from "react";
import {
  detectSources,
  getCostRates,
  updateCostRate,
  getDiagnostics,
  exportData,
  syncData,
  setSourceEnabled,
} from "../lib/tauri";
import type { SourceInfo, CostRate, Diagnostics } from "../lib/types";
import { formatBytes } from "../lib/format";
import { TOOL_COLORS } from "../lib/constants";
import { useSources } from "../contexts/SourceContext";

export default function Settings() {
  const { sources: contextSources, refreshSources } = useSources();
  const [sources, setSources] = useState<SourceInfo[]>(contextSources);
  const [rates, setRates] = useState<CostRate[]>([]);
  const [diagnostics, setDiagnostics] = useState<Diagnostics | null>(null);
  const [newModel, setNewModel] = useState("");

  useEffect(() => {
    detectSources().then(setSources);
    getCostRates().then(setRates);
    getDiagnostics().then(setDiagnostics);
  }, []);

  const handleRescan = async () => {
    const s = await detectSources();
    setSources(s);
    await syncData(true);
    const d = await getDiagnostics();
    setDiagnostics(d);
    await refreshSources();
  };

  const handleToggleSource = async (sourceId: string, enabled: boolean) => {
    await setSourceEnabled(sourceId, enabled);
    const updated = await detectSources();
    setSources(updated);
    await refreshSources();
  };

  const handleAddRate = async () => {
    if (!newModel.trim()) return;
    const rate: CostRate = {
      model: newModel.trim(),
      input_per_million: null,
      output_per_million: null,
      cache_read_per_million: null,
      cache_creation_per_million: null,
      effective_from: null,
    };
    await updateCostRate(rate);
    setRates(await getCostRates());
    setNewModel("");
  };

  const handleUpdateRate = async (
    model: string,
    field: keyof CostRate,
    value: string
  ) => {
    const existing = rates.find((r) => r.model === model);
    if (!existing) return;
    const updated = { ...existing, [field]: value ? parseFloat(value) : null };
    await updateCostRate(updated);
    setRates(await getCostRates());
  };

  const handleExport = async (format: string) => {
    const path = await exportData(format);
    alert(`Exported to: ${path}`);
  };

  const handleCopyDiagnostics = () => {
    if (!diagnostics) return;
    const syncLines = diagnostics.source_sync_times.map(
      (s) => `Last ${s.source_id} sync: ${s.last_sync_at}`
    );
    const text = [
      `Tally v${diagnostics.app_version}`,
      `Database: ${formatBytes(diagnostics.db_size_bytes)}`,
      `Sessions: ${diagnostics.total_sessions}`,
      `Requests: ${diagnostics.total_requests}`,
      ...syncLines,
    ].join("\n");
    navigator.clipboard.writeText(text);
  };

  return (
    <div>
      <h2 className="font-serif text-[28px] text-text-primary mb-8">
        Settings
      </h2>

      <div className="space-y-(--spacing-card-gap)">
        {/* Data Sources */}
        <Section title="Data Sources">
          <div className="space-y-2 mb-4">
            {sources.map((source) => (
              <SourceRow
                key={source.id}
                source={source}
                onToggle={(enabled) =>
                  handleToggleSource(source.id, enabled)
                }
              />
            ))}
          </div>
          <button
            onClick={handleRescan}
            className="px-4 py-1.5 text-sm bg-white border border-border rounded-(--radius-button) hover:bg-cream transition-all duration-300"
          >
            Re-scan Sources
          </button>
        </Section>

        {/* Cost Rates */}
        <Section title="Cost Rates">
          <p className="text-xs text-text-secondary mb-3">
            Enter your token pricing to see cost estimates on the dashboard.
          </p>
          <table className="w-full text-sm mb-3">
            <thead>
              <tr className="text-xs text-text-secondary text-left">
                <th className="pb-2">Model</th>
                <th className="pb-2">Input $/1M</th>
                <th className="pb-2">Output $/1M</th>
                <th className="pb-2">Cache Read $/1M</th>
                <th className="pb-2">Cache Write $/1M</th>
              </tr>
            </thead>
            <tbody>
              {rates.map((rate) => (
                <tr key={rate.model}>
                  <td className="py-1 pr-2 font-medium">{rate.model}</td>
                  {(
                    [
                      "input_per_million",
                      "output_per_million",
                      "cache_read_per_million",
                      "cache_creation_per_million",
                    ] as const
                  ).map((field) => (
                    <td key={field} className="py-1 pr-2">
                      <input
                        type="number"
                        step="0.01"
                        value={rate[field] ?? ""}
                        onChange={(e) =>
                          handleUpdateRate(rate.model, field, e.target.value)
                        }
                        className="w-20 px-2 py-1 rounded border border-border text-sm focus:outline-none focus:border-interactive"
                      />
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
          <div className="flex gap-2">
            <input
              type="text"
              placeholder="Model name"
              value={newModel}
              onChange={(e) => setNewModel(e.target.value)}
              className="px-3 py-1.5 rounded-(--radius-button) border border-border text-sm focus:outline-none focus:border-interactive"
            />
            <button
              onClick={handleAddRate}
              className="px-4 py-1.5 text-sm bg-white border border-border rounded-(--radius-button) hover:bg-cream transition-all duration-300"
            >
              Add Model
            </button>
          </div>
        </Section>

        {/* Export */}
        <Section title="Export">
          <div className="flex gap-2">
            <button
              onClick={() => handleExport("csv")}
              className="px-4 py-1.5 text-sm bg-white border border-border rounded-(--radius-button) hover:bg-cream transition-all duration-300"
            >
              Export as CSV
            </button>
            <button
              onClick={() => handleExport("json")}
              className="px-4 py-1.5 text-sm bg-white border border-border rounded-(--radius-button) hover:bg-cream transition-all duration-300"
            >
              Export as JSON
            </button>
          </div>
        </Section>

        {/* Diagnostics */}
        <Section title="Diagnostics">
          {diagnostics && (
            <div className="space-y-1 text-sm text-text-secondary">
              <p>Version: {diagnostics.app_version}</p>
              <p>Database: {formatBytes(diagnostics.db_size_bytes)}</p>
              <p>Sessions: {diagnostics.total_sessions.toLocaleString()}</p>
              <p>Requests: {diagnostics.total_requests.toLocaleString()}</p>
              {diagnostics.source_sync_times.map((s) => (
                <p key={s.source_id}>
                  Last {s.source_id} sync: {s.last_sync_at}
                </p>
              ))}
              <button
                onClick={handleCopyDiagnostics}
                className="mt-2 px-4 py-1.5 text-sm bg-white border border-border rounded-(--radius-button) hover:bg-cream transition-all duration-300"
              >
                Copy Diagnostics
              </button>
            </div>
          )}
        </Section>

        {/* About */}
        <Section title="About">
          <p className="text-sm text-text-secondary">
            Tally v{diagnostics?.app_version ?? "0.1.0"} · MIT License
          </p>
        </Section>
      </div>
    </div>
  );
}

function Section({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="bg-white rounded-(--radius-card) p-(--spacing-card-padding) border border-border">
      <h3 className="font-serif text-xl font-semibold text-text-primary mb-4">
        {title}
      </h3>
      {children}
    </div>
  );
}

function SourceRow({
  source,
  onToggle,
}: {
  source: SourceInfo;
  onToggle: (enabled: boolean) => void;
}) {
  const color = TOOL_COLORS[source.id] || "#8D8D83";

  return (
    <div className="flex items-center justify-between py-2">
      <div className="flex items-center gap-3">
        <div
          className="w-6 h-6 rounded flex items-center justify-center text-white text-xs font-bold"
          style={{ backgroundColor: color }}
        >
          {source.display_name[0]}
        </div>
        <div>
          <p className="text-sm font-medium text-text-primary">
            {source.display_name}
          </p>
          {source.path && (
            <p className="text-xs text-text-secondary font-mono">
              {source.path}
            </p>
          )}
        </div>
      </div>
      <div className="flex items-center gap-3">
        <span
          className={`text-sm ${source.detected ? "text-success" : "text-text-secondary"}`}
        >
          {source.detected
            ? `${source.session_count} sessions`
            : "Not installed"}
        </span>
        {source.detected && (
          <button
            onClick={() => onToggle(!source.enabled)}
            className={`relative w-9 h-5 rounded-full transition-colors duration-200 ${
              source.enabled ? "bg-success" : "bg-border"
            }`}
          >
            <div
              className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform duration-200 ${
                source.enabled ? "translate-x-4" : "translate-x-0.5"
              }`}
            />
          </button>
        )}
      </div>
    </div>
  );
}

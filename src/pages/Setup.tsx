import { useState, useEffect } from "react";
import { TallyLogo } from "../components/shared/TallyLogo";
import {
  detectSources,
  syncData,
  setSourceEnabled,
  onSyncProgress,
  onSyncComplete,
} from "../lib/tauri";
import type { SourceInfo, SyncResult } from "../lib/types";
import type { SyncProgress } from "../lib/tauri";
import { TOOL_COLORS } from "../lib/constants";

interface Props {
  onComplete: () => void;
}

type Step = "welcome" | "detecting" | "results" | "syncing" | "done";

export default function Setup({ onComplete }: Props) {
  const [step, setStep] = useState<Step>("welcome");
  const [sources, setSources] = useState<SourceInfo[]>([]);
  const [enabledIds, setEnabledIds] = useState<Set<string>>(new Set());
  const [syncResult, setSyncResult] = useState<SyncResult | null>(null);
  const [progress, setProgress] = useState<SyncProgress | null>(null);
  const [notInstalledMsg, setNotInstalledMsg] = useState<string | null>(null);

  // Listen for sync events
  useEffect(() => {
    const unlistenProgress = onSyncProgress((p) => {
      setProgress(p);
    });
    const unlistenComplete = onSyncComplete((result) => {
      setSyncResult(result);
      setStep("done");
    });
    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, []);

  const handleDetect = async () => {
    setStep("detecting");
    try {
      const detected = await detectSources();
      setSources(detected);
      // Auto-enable detected sources
      setEnabledIds(new Set(detected.filter((s) => s.detected).map((s) => s.id)));
      setStep("results");
    } catch {
      setStep("results");
    }
  };

  const handleToggle = async (source: SourceInfo) => {
    if (!source.detected) {
      setNotInstalledMsg(source.display_name);
      setTimeout(() => setNotInstalledMsg(null), 2000);
      return;
    }

    const next = new Set(enabledIds);
    if (next.has(source.id)) {
      next.delete(source.id);
    } else {
      next.add(source.id);
    }
    setEnabledIds(next);
  };

  const handleSync = async () => {
    setStep("syncing");
    setProgress(null);
    try {
      // Persist source preferences before syncing
      for (const source of sources) {
        await setSourceEnabled(source.id, enabledIds.has(source.id));
      }
      await syncData(true);
    } catch {
      setStep("done");
    }
  };

  const hasAnyEnabled = sources.some(
    (s) => s.detected && enabledIds.has(s.id)
  );

  // Build dynamic phase list for progress
  const syncPhases = sources
    .filter((s) => enabledIds.has(s.id) && s.detected)
    .map((s) => ({
      phase: `${s.id}_sync`,
      label: `Syncing ${s.display_name}`,
    }));

  return (
    <div className="min-h-screen bg-cream flex items-center justify-center">
      <div className="max-w-lg w-full mx-auto px-8">
        {step === "welcome" && (
          <div className="text-center">
            <div className="flex justify-center mb-6">
              <TallyLogo size={128} />
            </div>
            <h1 className="font-serif text-5xl text-text-primary mb-4">
              Tally
            </h1>
            <p className="text-text-secondary mb-2">
              Local LLM Token Usage Tracker
            </p>
            <p className="text-text-secondary text-sm mb-8">
              Tally reads your local AI tool data to show you how many tokens
              you're using. Everything stays on your machine — no cloud, no API
              keys.
            </p>
            <button
              onClick={handleDetect}
              className="px-6 py-2.5 bg-text-primary text-white rounded-(--radius-button) text-sm font-medium hover:opacity-90 transition-all duration-300"
            >
              Get Started
            </button>
          </div>
        )}

        {step === "detecting" && (
          <div className="text-center flex flex-col items-center">
            <TallyLogo size={120} className="mb-2" />
            <p className="text-text-secondary text-sm mt-2">
              Scanning for data sources...
            </p>
          </div>
        )}

        {step === "results" && (
          <div>
            <h2 className="font-serif text-2xl text-text-primary mb-2 text-center">
              Data Sources
            </h2>
            <p className="text-text-secondary text-sm mb-6 text-center">
              Select which tools to track
            </p>

            {notInstalledMsg && (
              <div className="mb-4 p-3 bg-warning/10 border border-warning/20 rounded-lg text-center">
                <p className="text-sm text-text-primary">
                  {notInstalledMsg} is not installed on this device
                </p>
              </div>
            )}

            <div className="space-y-2 mb-8">
              {sources.map((source) => (
                <SourceCard
                  key={source.id}
                  source={source}
                  enabled={enabledIds.has(source.id)}
                  onToggle={() => handleToggle(source)}
                />
              ))}
            </div>

            {hasAnyEnabled ? (
              <div className="text-center">
                <button
                  onClick={handleSync}
                  className="px-6 py-2.5 bg-text-primary text-white rounded-(--radius-button) text-sm font-medium hover:opacity-90 transition-all duration-300"
                >
                  Import Data
                </button>
              </div>
            ) : (
              <div className="text-center">
                <p className="text-text-secondary text-sm mb-4">
                  No sources selected. Install a supported AI tool or select a
                  detected source.
                </p>
                <button
                  onClick={onComplete}
                  className="px-6 py-2.5 bg-white text-text-primary border border-border rounded-(--radius-button) text-sm hover:bg-cream transition-all duration-300"
                >
                  Continue Anyway
                </button>
              </div>
            )}
          </div>
        )}

        {step === "syncing" && (
          <div className="text-center">
            <div className="flex justify-center mb-4">
              <TallyLogo size={160} />
            </div>
            <h2 className="font-serif text-2xl text-text-primary mb-6">
              Importing Data
            </h2>

            <div className="space-y-3 mb-8 text-left max-w-xs mx-auto">
              {syncPhases.map((sp) => (
                <ProgressStep
                  key={sp.phase}
                  label={sp.label}
                  status={getPhaseStatus(
                    sp.phase,
                    progress?.phase,
                    syncPhases.map((p) => p.phase)
                  )}
                />
              ))}
            </div>

            {progress && progress.phase !== "done" && (
              <div className="bg-white rounded-(--radius-card) p-4 border border-border">
                <p className="text-sm text-text-primary font-medium">
                  {progress.message}
                </p>
                {(progress.sessions_so_far > 0 ||
                  progress.requests_so_far > 0) && (
                  <p className="text-xs text-text-secondary mt-1">
                    {progress.sessions_so_far} sessions ·{" "}
                    {progress.requests_so_far} requests imported so far
                  </p>
                )}
              </div>
            )}
          </div>
        )}

        {step === "done" && (
          <div className="text-center">
            <div className="w-12 h-12 rounded-full bg-success/10 flex items-center justify-center mx-auto mb-4">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
                <path
                  d="M20 6L9 17l-5-5"
                  stroke="#2D9E73"
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
            </div>
            <h2 className="font-serif text-2xl text-text-primary mb-2">
              All set!
            </h2>
            {syncResult && (
              <p className="text-text-secondary text-sm mb-6">
                Imported {syncResult.new_sessions} sessions and{" "}
                {syncResult.new_requests} requests
              </p>
            )}
            <button
              onClick={onComplete}
              className="px-6 py-2.5 bg-text-primary text-white rounded-(--radius-button) text-sm font-medium hover:opacity-90 transition-all duration-300"
            >
              Open Dashboard
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

function getPhaseStatus(
  phase: string,
  currentPhase: string | undefined,
  phaseOrder: string[]
): "pending" | "active" | "done" {
  if (!currentPhase) return "pending";
  if (currentPhase === "done") return "done";
  const phaseIdx = phaseOrder.indexOf(phase);
  const currentIdx = phaseOrder.indexOf(currentPhase);
  if (currentIdx < 0) return "pending"; // unknown phase
  if (currentIdx > phaseIdx) return "done";
  if (currentIdx === phaseIdx) return "active";
  return "pending";
}

function ProgressStep({
  label,
  status,
}: {
  label: string;
  status: "pending" | "active" | "done";
}) {
  return (
    <div className="flex items-center gap-3">
      {status === "done" && (
        <div className="w-5 h-5 rounded-full bg-success flex items-center justify-center shrink-0">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
            <path
              d="M10 3L4.5 8.5 2 6"
              stroke="white"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </div>
      )}
      {status === "active" && (
        <div className="w-5 h-5 shrink-0">
          <div className="w-5 h-5 border-2 border-text-secondary border-t-text-primary rounded-full animate-spin" />
        </div>
      )}
      {status === "pending" && (
        <div className="w-5 h-5 rounded-full border-2 border-border shrink-0" />
      )}
      <span
        className={`text-sm ${
          status === "active"
            ? "text-text-primary font-medium"
            : status === "done"
              ? "text-success"
              : "text-text-secondary"
        }`}
      >
        {label}
      </span>
    </div>
  );
}

function SourceCard({
  source,
  enabled,
  onToggle,
}: {
  source: SourceInfo;
  enabled: boolean;
  onToggle: () => void;
}) {
  const color = TOOL_COLORS[source.id] || "#8D8D83";

  return (
    <button
      onClick={onToggle}
      className={`w-full bg-white rounded-(--radius-card) p-4 border flex items-center justify-between transition-all duration-200 ${
        source.detected
          ? enabled
            ? "border-text-primary"
            : "border-border hover:border-text-secondary"
          : "border-border opacity-50 cursor-default"
      }`}
    >
      <div className="flex items-center gap-3">
        <div
          className="w-8 h-8 rounded-lg flex items-center justify-center text-white text-sm font-bold"
          style={{ backgroundColor: color }}
        >
          {source.display_name[0]}
        </div>
        <span className="text-sm font-medium text-text-primary">
          {source.display_name}
        </span>
      </div>
      <div className="flex items-center gap-3">
        <span className="text-sm text-text-secondary">
          {source.detected
            ? `${source.session_count} sessions`
            : "Not installed"}
        </span>
        {/* Checkbox */}
        <div
          className={`w-5 h-5 rounded border-2 flex items-center justify-center transition-all duration-200 ${
            enabled
              ? "bg-text-primary border-text-primary"
              : "border-border bg-white"
          }`}
        >
          {enabled && (
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path
                d="M10 3L4.5 8.5 2 6"
                stroke="white"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
          )}
        </div>
      </div>
    </button>
  );
}

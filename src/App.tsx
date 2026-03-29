import { useState, useEffect, useRef } from "react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { AppLayout } from "./components/layout/AppLayout";
import { getSyncStatus, detectSources, syncData } from "./lib/tauri";
import { SourceProvider } from "./contexts/SourceContext";
import { UpdateChecker } from "./components/shared/UpdateChecker";
import type { SourceInfo } from "./lib/types";
import Home from "./pages/Home";
import ToolDashboard from "./pages/ToolDashboard";
import Sessions from "./pages/Sessions";
import Settings from "./pages/Settings";
import Setup from "./pages/Setup";

function App() {
  const [showSetup, setShowSetup] = useState<boolean | null>(null);
  const [sources, setSources] = useState<SourceInfo[]>([]);
  const initialSyncDone = useRef(false);

  useEffect(() => {
    // Check if we've synced before and load sources
    Promise.all([getSyncStatus(), detectSources()])
      .then(([states, detectedSources]) => {
        setSources(detectedSources);
        const needsSetup = states.length === 0;
        setShowSetup(needsSetup);
        // Auto-sync on app open (once, only if not first launch)
        if (!needsSetup && !initialSyncDone.current) {
          initialSyncDone.current = true;
          syncData(false).catch(() => {});
        }
      })
      .catch(() => {
        setShowSetup(true);
      });
  }, []);

  if (showSetup === null) {
    return (
      <div className="min-h-screen bg-cream flex items-center justify-center">
        <div className="animate-spin w-6 h-6 border-2 border-text-secondary border-t-text-primary rounded-full" />
      </div>
    );
  }

  if (showSetup) {
    return (
      <Setup
        onComplete={async () => {
          const detectedSources = await detectSources();
          setSources(detectedSources);
          setShowSetup(false);
        }}
      />
    );
  }

  const enabledSources = sources.filter((s) => s.enabled);

  return (
    <SourceProvider initialSources={sources}>
      <UpdateChecker />
      <MemoryRouter>
        <AppLayout>
          <Routes>
            <Route path="/" element={<Home />} />
            {enabledSources.map((source) => (
              <Route
                key={source.id}
                path={`/tool/${source.id}`}
                element={
                  <ToolDashboard
                    toolId={source.id}
                    displayName={source.display_name}
                  />
                }
              />
            ))}
            <Route path="/sessions" element={<Sessions />} />
            <Route path="/settings" element={<Settings />} />
          </Routes>
        </AppLayout>
      </MemoryRouter>
    </SourceProvider>
  );
}

export default App;

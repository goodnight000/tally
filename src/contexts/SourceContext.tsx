import { createContext, useContext, useState, useCallback } from "react";
import type { SourceInfo } from "../lib/types";
import { detectSources } from "../lib/tauri";

interface SourceContextValue {
  sources: SourceInfo[];
  refreshSources: () => Promise<void>;
}

const SourceContext = createContext<SourceContextValue>({
  sources: [],
  refreshSources: async () => {},
});

export function SourceProvider({
  initialSources,
  children,
}: {
  initialSources: SourceInfo[];
  children: React.ReactNode;
}) {
  const [sources, setSources] = useState<SourceInfo[]>(initialSources);

  const refreshSources = useCallback(async () => {
    try {
      const updated = await detectSources();
      setSources(updated);
    } catch {
      // Keep current sources on error
    }
  }, []);

  return (
    <SourceContext.Provider value={{ sources, refreshSources }}>
      {children}
    </SourceContext.Provider>
  );
}

export function useSources() {
  return useContext(SourceContext);
}

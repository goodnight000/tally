import { useState, useEffect } from "react";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

type UpdateState =
  | { status: "idle" }
  | { status: "available"; version: string }
  | { status: "downloading"; progress: number }
  | { status: "ready" }
  | { status: "error"; message: string };

export function UpdateChecker() {
  const [state, setState] = useState<UpdateState>({ status: "idle" });
  const [dismissed, setDismissed] = useState(false);

  useEffect(() => {
    let cancelled = false;

    async function checkForUpdate() {
      try {
        const update = await check();
        if (cancelled) return;
        if (update) {
          setState({ status: "available", version: update.version });
        }
      } catch {
        // Silently ignore update check failures (offline, no releases yet, etc.)
      }
    }

    checkForUpdate();
    return () => {
      cancelled = true;
    };
  }, []);

  async function installUpdate() {
    try {
      setState({ status: "downloading", progress: 0 });
      const update = await check();
      if (!update) return;

      let totalLength = 0;
      let downloaded = 0;

      await update.downloadAndInstall((event) => {
        if (event.event === "Started" && event.data.contentLength) {
          totalLength = event.data.contentLength;
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          if (totalLength > 0) {
            setState({
              status: "downloading",
              progress: Math.round((downloaded / totalLength) * 100),
            });
          }
        } else if (event.event === "Finished") {
          setState({ status: "ready" });
        }
      });

      setState({ status: "ready" });
    } catch (e) {
      setState({
        status: "error",
        message: e instanceof Error ? e.message : "Update failed",
      });
    }
  }

  async function handleRelaunch() {
    await relaunch();
  }

  if (state.status === "idle" || dismissed) return null;

  return (
    <div className="fixed bottom-6 right-6 z-50 max-w-sm rounded-card border border-border bg-cream shadow-lg p-4">
      {state.status === "available" && (
        <>
          <div className="flex items-start justify-between gap-3">
            <div>
              <p className="font-semibold text-text-primary text-sm">
                Update available
              </p>
              <p className="text-text-secondary text-xs mt-0.5">
                Tally v{state.version} is ready to install.
              </p>
            </div>
            <button
              onClick={() => setDismissed(true)}
              className="text-text-secondary hover:text-text-primary text-lg leading-none -mt-1 cursor-pointer"
            >
              &times;
            </button>
          </div>
          <div className="flex gap-2 mt-3">
            <button
              onClick={installUpdate}
              className="px-3 py-1.5 text-xs font-medium rounded-button bg-terracotta text-cream hover:opacity-90 cursor-pointer"
            >
              Install &amp; restart
            </button>
            <button
              onClick={() => setDismissed(true)}
              className="px-3 py-1.5 text-xs font-medium rounded-button border border-border text-text-secondary hover:text-text-primary cursor-pointer"
            >
              Later
            </button>
          </div>
        </>
      )}

      {state.status === "downloading" && (
        <div>
          <p className="font-semibold text-text-primary text-sm">
            Downloading update...
          </p>
          <div className="mt-2 h-1.5 w-full rounded-full bg-border overflow-hidden">
            <div
              className="h-full rounded-full bg-terracotta transition-all duration-300"
              style={{ width: `${state.progress}%` }}
            />
          </div>
          <p className="text-text-secondary text-xs mt-1">{state.progress}%</p>
        </div>
      )}

      {state.status === "ready" && (
        <div>
          <p className="font-semibold text-text-primary text-sm">
            Update installed
          </p>
          <p className="text-text-secondary text-xs mt-0.5">
            Restart Tally to apply the update.
          </p>
          <button
            onClick={handleRelaunch}
            className="mt-3 px-3 py-1.5 text-xs font-medium rounded-button bg-terracotta text-cream hover:opacity-90 cursor-pointer"
          >
            Restart now
          </button>
        </div>
      )}

      {state.status === "error" && (
        <div className="flex items-start justify-between gap-3">
          <div>
            <p className="font-semibold text-text-primary text-sm">
              Update failed
            </p>
            <p className="text-text-secondary text-xs mt-0.5">
              {state.message}
            </p>
          </div>
          <button
            onClick={() => setDismissed(true)}
            className="text-text-secondary hover:text-text-primary text-lg leading-none -mt-1 cursor-pointer"
          >
            &times;
          </button>
        </div>
      )}
    </div>
  );
}

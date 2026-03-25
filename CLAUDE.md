# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What is Tally

Tally is a cross-platform desktop app that tracks token usage from Claude Code and Codex CLI. It reads local data files from both tools (`~/.claude/` and `~/.codex/`), normalizes them into a unified SQLite database (`~/.tally/tally.sqlite`), and displays a dashboard. No cloud, no API keys — everything is local and read-only against source data.

## Commands

```bash
# Development (starts both Vite dev server and Tauri window)
npm run tauri dev

# Build production binary
npm run tauri build

# Frontend only (no Tauri window, for UI work at localhost:1420)
npm run dev

# Type-check frontend
npx tsc --noEmit

# Run Rust tests
cd src-tauri && cargo test

# Run a single Rust test
cd src-tauri && cargo test <test_name>
```

## Architecture

**Tauri v2 app** — Rust backend + React/TypeScript frontend communicating via Tauri's `invoke` IPC.

### Backend (src-tauri/src/)

- `lib.rs` — App entry point. Registers all Tauri commands, initializes DB, manages `AppState` (Arc<Mutex<Connection>>).
- `db/` — SQLite layer using rusqlite. `mod.rs` handles init + WAL mode. `migrations.rs` for schema versioning. `queries.rs` for read queries. `models.rs` for Rust structs. `cost_defaults.rs` for auto-populating model pricing.
- `parsers/` — Data ingestion from source tools:
  - `codex/sqlite.rs` — Reads Codex's `state_*.sqlite` (discovers highest version dynamically). Opens read-only.
  - `codex/jsonl.rs` — Parses `~/.codex/sessions/` JSONL files for per-request token data.
  - `claude/jsonl.rs` — Parses `~/.claude/projects/<path>/<session>.jsonl` for usage data.
  - `claude/stats_cache.rs` — Reads `~/.claude/stats-cache.json` for fast initial data.
  - `claude/session_index.rs` — Reads `~/.claude/sessions/<pid>.json` for session metadata.
  - `normalize.rs` — Path normalization (Claude encodes paths with dashes, Codex uses absolute paths).
- `commands/` — Tauri command handlers exposed to frontend:
  - `sync.rs` — `detect_sources`, `sync_data` (async via Tauri events), `get_sync_status`
  - `dashboard.rs` — Aggregation queries for stats, daily usage, model breakdown, heatmap, etc.
  - `sessions.rs` — Paginated session list with filtering, session detail with child requests
  - `settings.rs` — Cost rates CRUD, diagnostics, data export

### Frontend (src/)

- `lib/tauri.ts` — Typed wrappers around all `invoke()` calls. This is the single interface to the backend.
- `lib/types.ts` — All TypeScript interfaces matching Rust serialized structs.
- `App.tsx` — Root component. Uses `MemoryRouter`. Shows Setup wizard on first launch, then AppLayout with routes.
- `pages/` — Route pages: Home (unified dashboard), ClaudeCode, Codex (filtered views), Sessions (table with filters), Settings.
- `components/layout/` — AppLayout, Sidebar, TopBar.
- `components/dashboard/` — StatCards, SessionFeed, TopProjects, BiggestSessions, ModelBreakdown, Heatmap.
- `components/charts/` — DailyUsageChart (Recharts).
- `components/shared/` — Reusable: StatCard, Badge, AnimatedNumber, EmptyState, DateRangePicker, ToolIcon.
- `hooks/` — `useDashboard.ts`, `useSessions.ts`, `useSync.ts`.

### Key patterns

- **Sync flow**: Rust reads source data → normalizes → `INSERT OR IGNORE` into unified DB → emits Tauri events (`sync-progress`, `sync-complete`) → frontend listens and re-renders.
- **Incremental sync**: Watermarks stored in `sync_state` table. Codex uses `updated_at` timestamps; Claude uses byte offsets per JSONL file.
- **Deterministic IDs**: Session IDs are `codex:<thread_id>` or `claude:<session_id>`. Request IDs include line numbers/UUIDs. Prevents duplicates.
- **Codex schema flexibility**: Uses `PRAGMA table_info` to discover available columns since Codex DB schema evolves across versions.

## Design tokens

Defined in `src/styles/index.css` via Tailwind v4 `@theme`:
- Background: `cream` (#FFFFEB) — light mode only, no dark mode
- Claude accent: `terracotta` (#CC785C), Codex accent: `periwinkle` (#7B8CEA)
- Fonts: Figtree (body/UI), EB Garamond (headings/stat numbers)
- Spacing: sidebar 200px, content padding 32px, card padding 24px, card gap 16px

## Important constraints

- **Read-only**: Never write to `~/.codex/` or `~/.claude/`. Tally only reads source data.
- **Data minimization**: Only extract token counts, models, timestamps, and project metadata. Never store conversation content.
- **Concurrent access**: Open Codex SQLite in read-only mode. Handle locked DB gracefully (skip and retry on next sync).
- **No network**: No telemetry, no analytics, no cloud services. Fully local.

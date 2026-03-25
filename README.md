# Tally

A desktop app that tracks your token usage from **Claude Code** and **Codex CLI**. It reads local data files from both tools, normalizes them into a unified SQLite database, and displays a dashboard with charts, session history, and cost breakdowns.

No cloud, no API keys — everything runs locally.

![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)
![Built with](https://img.shields.io/badge/built%20with-Tauri%20v2-blue)

## Features

- **Unified dashboard** — Combined view of token usage across Claude Code, Codex CLI, and other supported tools
- **Per-tool views** — Filtered dashboards for each tool
- **Session browser** — Paginated session list with filters, expandable to see individual requests
- **Daily usage charts** — Track spend and token consumption over time
- **Model breakdown** — See which models you're using and their costs
- **Activity heatmap** — Visualize coding patterns
- **Cost tracking** — Customizable per-model cost rates
- **Data export** — Export your data as JSON
- **Fully local** — No network calls, no telemetry, read-only access to source data

## Supported Tools

| Tool | Data Source |
|------|-------------|
| Claude Code | `~/.claude/` (JSONL session files, stats cache, session index) |
| Codex CLI | `~/.codex/` (SQLite DB + JSONL session files) |

## Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install) (stable, 2021 edition)
- Tauri v2 system dependencies — see the [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/) for your OS

## Getting Started

```bash
# Clone the repo
git clone https://github.com/your-username/tally.git
cd tally

# Install frontend dependencies
npm install

# Run in development mode (starts Vite dev server + Tauri window)
npm run tauri dev
```

On first launch, Tally will detect available data sources and walk you through a setup wizard.

## Commands

```bash
# Development (Vite + Tauri window)
npm run tauri dev

# Build production binary
npm run tauri build

# Frontend only (no Tauri window, for UI work at localhost:1420)
npm run dev

# Type-check frontend
npx tsc --noEmit

# Run Rust tests
cd src-tauri && cargo test
```

## Project Structure

```
tally/
├── src/                    # React/TypeScript frontend
│   ├── pages/              # Home, ToolDashboard, Sessions, Settings, Setup
│   ├── components/         # Dashboard widgets, charts, layout, shared UI
│   ├── hooks/              # useDashboard, useSessions, useSync
│   ├── lib/                # Tauri IPC wrappers (tauri.ts) and types (types.ts)
│   └── styles/             # Tailwind v4 theme tokens
├── src-tauri/              # Rust backend
│   └── src/
│       ├── lib.rs          # App entry, command registration, state management
│       ├── commands/       # Tauri commands: sync, dashboard, sessions, settings
│       ├── db/             # SQLite: migrations, queries, models, cost defaults
│       └── parsers/        # Data ingestion from Claude Code, Codex, and others
├── package.json
└── CLAUDE.md
```

## How It Works

1. **Detect** — Scans for `~/.claude/` and `~/.codex/` directories
2. **Parse** — Reads JSONL files and SQLite databases (read-only, never writes to source data)
3. **Normalize** — Extracts token counts, models, timestamps, and project metadata into `~/.tally/tally.sqlite`
4. **Display** — React frontend queries the unified DB via Tauri IPC and renders the dashboard
5. **Incremental sync** — Tracks watermarks (byte offsets, timestamps) so subsequent syncs only process new data

## Tech Stack

- **Framework**: [Tauri v2](https://v2.tauri.app/) (Rust + Web)
- **Frontend**: React 19, TypeScript, Tailwind CSS v4, Recharts
- **Backend**: Rust, rusqlite (bundled SQLite), serde, chrono
- **Build**: Vite 8, npm

## Privacy

Tally is designed with privacy in mind:

- **No network access** — Zero telemetry, analytics, or cloud services
- **Read-only** — Never writes to Claude Code or Codex data directories
- **Data minimization** — Only stores token counts, model names, timestamps, and project paths. Never stores conversation content.
- **Local storage** — All data lives in `~/.tally/tally.sqlite`

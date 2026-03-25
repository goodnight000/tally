# Tally — Product Requirements Document

**AI CLI Token Usage Tracker**
Open source cross-platform desktop app for tracking Claude Code and Codex CLI token consumption.

---

## 1. Problem

Developers using Claude Code and Codex CLI have no unified way to see how many tokens they're consuming. Both tools store usage data locally in different formats, but there's no dashboard to visualize it. Users want to understand their AI coding tool usage patterns, costs, and efficiency — all in one place.

## 2. Solution

Tally is a lightweight desktop app (Mac + Windows) that reads local CLI data and presents it as a clean, modern dashboard. It requires no cloud services, no API keys, and no configuration — everything runs locally by reading data that both tools already store on disk.

## 3. Target User

Developers who actively use Claude Code and/or Codex CLI and want visibility into their token consumption.

## 4. Tech Stack

| Layer | Technology |
|-------|------------|
| Framework | Tauri v2 (Rust + WebView) |
| Frontend | React + TypeScript + Tailwind CSS |
| Charts | Recharts (or similar lightweight React charting lib) |
| Local DB | SQLite (via rusqlite in the Rust backend) |
| Fonts | Figtree (sans-serif), EB Garamond (serif) |
| Build | GitHub Actions for cross-platform releases |
| Distribution | `.dmg` (Mac), `.msi` (Windows), GitHub Releases |

### Why Tauri
- ~5MB binary vs Electron's ~150MB
- Native OS integration (window management, file system access)
- Rust backend is ideal for reading SQLite and parsing JSONL files performantly
- Web frontend allows precise control over the Wispr-inspired design
- Cross-platform from one codebase
- Open source friendly (MIT)

## 5. Data Sources

Both tools already store rich token usage data locally. Tally reads this data directly — no setup, no configuration, no shell profile modifications.

### 5.1 Codex CLI (works out of the box)

**Session-level data:**
- Source: `~/.codex/state_*.sqlite` → `threads` table (dynamically discover highest version number, e.g. `state_5.sqlite`, `state_6.sqlite`)
- Fields: `tokens_used`, `model` (nullable — may be NULL for older sessions), `model_provider`, `created_at`, `updated_at`, `cwd`, `git_sha`, `git_branch`, `git_origin_url`, `cli_version`
- Important: the `source` field can be `"cli"`, `"vscode"`, or a JSON object for subagent threads. ~64% of threads may be subagent spawns. These must be distinguished from top-level sessions to avoid double-counting tokens.

**Per-request granular data:**
- Source: `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`
- Token fields in `token_count` events: `input_tokens`, `cached_input_tokens`, `output_tokens`, `reasoning_output_tokens`, `total_tokens`
- Also includes: `model_context_window`, `rate_limits`
- The `session_meta` event in each JSONL file contains the model name, which can be used as a fallback when the SQLite `model` column is NULL.

**Handling schema evolution:**
- Codex has evolved through database versions 1-5+ and added columns incrementally (`model`, `agent_nickname`, `reasoning_effort`, etc.)
- All `SELECT` queries must gracefully handle missing columns (use `PRAGMA table_info` to discover available columns)
- Discover database file dynamically: scan `~/.codex/` for `state_*.sqlite` and use the highest version number

**Custom paths:**
- Check `CODEX_HOME` environment variable and `XDG_DATA_HOME` before falling back to `~/.codex/`

### 5.2 Claude Code (works out of the box)

**Per-session JSONL files:**
- Source: `~/.claude/projects/<encoded-project-path>/<session-id>.jsonl`
- Each assistant message contains a `usage` object with: `input_tokens`, `output_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`
- Also includes: `model`, `sessionId`, `timestamp` (ISO 8601 UTC), `cwd`, `gitBranch`, `version`

**Aggregate stats cache:**
- Source: `~/.claude/stats-cache.json`
- Contains daily activity totals, per-model token breakdowns, and cumulative usage
- Fields per model: `inputTokens`, `outputTokens`, `cacheReadInputTokens`, `cacheCreationInputTokens`
- Useful for fast initial dashboard render before full JSONL parsing completes

**Session index:**
- Source: `~/.claude/sessions/<pid>.json`
- Maps PIDs to session IDs, working directories, and start times

**Data minimization:**
- Claude Code JSONL files contain full conversation content including code. Tally must only extract token counts, model, timestamps, and project metadata. Never store or cache conversation content.

### 5.3 Tally's Own Database

A unified SQLite database (`~/.tally/tally.sqlite`) that normalizes data from both sources into a common schema.

**Schema:**

```sql
-- Database version tracking for migrations
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Sync watermarks for incremental updates
CREATE TABLE sync_state (
    source TEXT PRIMARY KEY,                    -- 'codex' | 'claude'
    last_sync_at TEXT NOT NULL,                 -- ISO 8601 timestamp
    last_codex_thread_updated_at INTEGER,       -- Unix timestamp from threads table
    last_claude_file_offsets TEXT,              -- JSON map of {filepath: byte_offset}
    last_codex_db_version INTEGER              -- e.g. 5 for state_5.sqlite
);

-- Normalized session data from both tools
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,                        -- deterministic: 'codex:<thread_id>' or 'claude:<session_id>'
    tool TEXT NOT NULL,                         -- 'claude' | 'codex'
    source TEXT,                                -- 'cli' | 'vscode' | 'subagent'
    parent_session_id TEXT,                     -- for Codex subagent threads
    model TEXT,                                 -- may be NULL for older Codex sessions
    title TEXT,                                 -- Codex provides thread titles
    start_time TEXT NOT NULL,                   -- ISO 8601 in UTC
    end_time TEXT,                              -- ISO 8601 in UTC
    project_path TEXT,                          -- normalized absolute path
    project_name TEXT,                          -- derived: last path component or repo name
    git_branch TEXT,
    git_sha TEXT,
    git_origin_url TEXT,
    cli_version TEXT,
    total_input_tokens INTEGER DEFAULT 0,
    total_output_tokens INTEGER DEFAULT 0,
    total_cache_read_tokens INTEGER DEFAULT 0,
    total_cache_creation_tokens INTEGER DEFAULT 0,
    total_reasoning_tokens INTEGER DEFAULT 0,
    total_tokens INTEGER DEFAULT 0,
    FOREIGN KEY (parent_session_id) REFERENCES sessions(id)
);

-- Per-request granular token data
CREATE TABLE requests (
    id TEXT PRIMARY KEY,                        -- deterministic: 'codex:<thread_id>:<line_num>' or 'claude:<session_id>:<msg_uuid>'
    session_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,                    -- ISO 8601 in UTC
    model TEXT,
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    cache_read_tokens INTEGER DEFAULT 0,
    cache_creation_tokens INTEGER DEFAULT 0,
    reasoning_tokens INTEGER DEFAULT 0,
    total_tokens INTEGER DEFAULT 0,
    duration_ms INTEGER,
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

-- User-configured cost rates per model
CREATE TABLE cost_rates (
    model TEXT PRIMARY KEY,
    input_per_million REAL,                     -- USD per 1M input tokens
    output_per_million REAL,                    -- USD per 1M output tokens
    cache_read_per_million REAL,               -- USD per 1M cache read tokens
    cache_creation_per_million REAL,            -- USD per 1M cache creation tokens
    effective_from TEXT                         -- ISO 8601 date
);

CREATE INDEX idx_sessions_tool ON sessions(tool);
CREATE INDEX idx_sessions_start_time ON sessions(start_time);
CREATE INDEX idx_sessions_project_name ON sessions(project_name);
CREATE INDEX idx_requests_session_id ON requests(session_id);
CREATE INDEX idx_requests_timestamp ON requests(timestamp);
```

**De-duplication:**
- All IDs are deterministic, derived from source data (not auto-increment)
- Session IDs: `codex:<codex_thread_id>` or `claude:<claude_session_id>`
- Request IDs: `codex:<thread_id>:<jsonl_line_number>` or `claude:<session_id>:<message_uuid>`
- `INSERT OR IGNORE` prevents duplicates on re-sync

**Database migrations:**
- `schema_version` table tracks the current schema version
- On launch, Tally checks version and runs any pending migration scripts sequentially
- Migrations are embedded in the Rust binary (not external SQL files)

**Project path normalization:**
- Codex stores `cwd` as absolute paths (e.g. `/Users/charles/repos/myproject`)
- Claude Code encodes paths with dashes (e.g. `-Users-charles-repos-myproject`)
- Tally normalizes both to absolute paths and derives `project_name` from: `git_origin_url` repo name if available, otherwise the last path component

**Data retention:**
- No automatic pruning in v1. Users can export and manually delete via Settings.
- Monitor `~/.tally/tally.sqlite` file size in Settings page.
- Future: configurable retention period (e.g. keep last 6 months)

## 6. First-Launch Setup

Since both tools store data locally, setup is automatic detection — no user configuration needed.

1. **Welcome screen** — explains what Tally tracks and that it reads local data only (no cloud, no API keys)
2. **Auto-detection** — scans for both data sources:
   - Codex: checks `~/.codex/state_*.sqlite`. Shows "Found X sessions" or "Not detected — install Codex CLI to start tracking"
   - Claude Code: checks `~/.claude/projects/`. Shows "Found X sessions" or "Not detected — install Claude Code to start tracking"
3. **Initial sync** — imports all available data with a progress indicator (may take 10-30 seconds for large histories)
4. **Done** — shows the dashboard with whatever data is available

Post-setup, if a tool is installed later, Tally detects it automatically on next sync. Settings page also has a "Re-scan data sources" button.

## 7. Dashboard Layout

### 7.1 Structure

```
┌──────────────────────────────────────────────────────┐
│  [Logo] Tally                    [Refresh] [Settings]│
├────────────┬─────────────────────────────────────────┤
│            │                                         │
│  Home      │  Welcome back                           │
│  Claude    │  [streak] [tokens today] [sessions]     │
│  Codex     │                                         │
│  Sessions  │  ┌─────────────────────────────────┐    │
│  Settings  │  │  Daily Usage Chart (stacked)    │    │
│            │  └─────────────────────────────────┘    │
│            │                                         │
│            │  ┌──────────┐ ┌──────────┐ ┌────────┐  │
│            │  │ Input vs │ │ Cached   │ │ Cost   │  │
│            │  │ Output   │ │ Savings  │ │ Est.   │  │
│            │  └──────────┘ └──────────┘ └────────┘  │
│            │                                         │
│            │  ┌─────────────────────────────────┐    │
│            │  │  Session List / Activity Feed   │    │
│            │  └─────────────────────────────────┘    │
│            │                                         │
└────────────┴─────────────────────────────────────────┘
```

### 7.2 Pages

**Home** — unified dashboard with all stats across both tools

**Claude Code** — filtered view showing only Claude Code usage, with model breakdown (Opus, Sonnet, Haiku)

**Codex** — filtered view showing only Codex usage, with model breakdown (GPT-5.4, 5.4-mini, etc.)

**Sessions** — full searchable/sortable table of all sessions with drill-down

**Settings** — configure data sources, cost rates, export data

### 7.3 Home Page Components

**Top bar quick stats** (like Wispr's streak/words/WPM badges):
- Usage streak (consecutive days)
- Tokens today
- Sessions today

**Daily usage chart:**
- Stacked area/bar chart, Claude Code (terracotta) + Codex (periwinkle)
- Toggle between: day / week / month views
- Toggle between: tokens / estimated cost
- Hover: tooltip showing exact values for that day
- Click on a day: filters session activity feed below to that day

**Three stat cards row:**
- Input vs Output ratio — donut or split bar
- Cache efficiency — cached tokens as % of total input, estimated savings (cache read tokens vs cache creation tokens shown separately)
- Cost estimate — daily/weekly/monthly based on user-configured rates

**Model breakdown:**
- Horizontal bar chart showing token distribution by model
- Both tools combined, color-coded

**Day-of-week heatmap:**
- 7-column heat grid showing usage intensity by day
- Expandable to hour-of-day view

**Efficiency metrics row:**
- Output-to-input ratio
- Reasoning token % (Codex)
- Cache hit rate (Claude Code)
- Avg tokens per session

**Trends:**
- This week vs last week comparison (% change arrows)
- Rolling 7-day average line

**Session activity feed:**
- Most recent sessions (top-level only — subagent threads rolled up into parent), Wispr-style timeline layout
- Timestamp | Tool icon | Model | Token count | Project name
- Click to expand inline: shows per-request breakdown as a nested list with individual token counts, model, and timestamps. Subagent threads shown as indented children.

**Top projects:**
- Bar chart of token usage grouped by project/repo (using normalized `project_name`)

**Biggest sessions:**
- Top 10 most token-heavy sessions, ranked

### 7.4 Sessions Page

**Filters:**
- Date range picker (preset: today, last 7 days, last 30 days, all time, custom range)
- Tool: All / Claude Code / Codex
- Model: multi-select dropdown populated from available data
- Project: multi-select dropdown populated from available data
- Token threshold: min/max token count
- Source: All / CLI / VS Code / Subagent

**Search:**
- Free-text search across session title, project name, git branch

**Table columns:**
- Timestamp, Tool, Model, Source, Project, Branch, Input Tokens, Output Tokens, Cached Tokens, Total Tokens, Duration
- All columns sortable (click header to sort asc/desc)

**Drill-down:**
- Click a row to expand inline, showing per-request breakdown
- Subagent threads shown as expandable children under their parent session

### 7.5 Empty States

Every dashboard component has a defined empty state:

| Component | Empty State |
|-----------|-------------|
| Top bar stats | "0 tokens today · 0 sessions" (no error, just zeroes) |
| Daily usage chart | Flat line at zero with muted text: "No usage data yet" |
| Stat cards | Show "—" with label "No data for this period" |
| Model breakdown | "No model data available" |
| Heatmap | Gray grid with "Start using Claude Code or Codex to see patterns" |
| Session feed | Illustration + "No sessions recorded yet. Usage will appear here automatically." |
| Top projects | "No project data yet" |
| Biggest sessions | "No sessions recorded" |

When only one tool has data, the other tool's sections show data as zero (not hidden) so the user knows tracking is available for both.

### 7.6 Loading States

- **Initial sync (first launch):** full-screen progress indicator with "Importing X sessions..." count and progress bar
- **Incremental refresh:** subtle spinner on the refresh button icon (300ms spin animation). Dashboard shows stale cached data immediately — new data merges in when sync completes. No full-screen loading.
- **Refresh error:** toast notification at bottom: "Failed to read Codex data — file may be in use. Will retry on next refresh."
- **Refresh timeout:** 30-second timeout. If sync hasn't completed, show stale data with a warning badge on the refresh button.

## 8. Design Specification

### 8.1 Colors

| Role | Value |
|------|-------|
| Page background | `#FFFFEB` (warm cream) |
| Sidebar background | `#FFFFFF` |
| Card background | `#FFFFFF` |
| Primary text | `#1A1A1A` |
| Secondary text | `#8D8D83` |
| Claude Code accent | `#CC785C` (terracotta) |
| Codex accent | `#7B8CEA` (periwinkle) |
| Borders/dividers | `#E8E8E0` (warm light gray) |
| Hover/interactive | `#4D65FF` (soft blue) |
| Success/positive | `#2D9E73` |
| Warning/negative | `#D97706` |

**System theme:** Tally always renders in light mode regardless of OS dark mode setting. The warm cream palette is core to the brand identity and cannot be meaningfully adapted to dark mode without losing the Wispr-inspired aesthetic.

### 8.2 Typography

| Element | Font | Weight | Size |
|---------|------|--------|------|
| Nav items | Figtree | 400 | 14px |
| Body text | Figtree | 400 | 14px |
| Labels / captions | Figtree | 400 | 12px |
| Section headings | EB Garamond | 600 | 20px |
| Page title ("Welcome back") | EB Garamond | 400 | 28px |
| Big stat numbers | EB Garamond | 400 italic | 36px |
| Stat card labels | Figtree | 400 | 12px |
| Badge text | Figtree | 600 | 13px |

### 8.3 Spacing & Radius

- Sidebar width: 200px (collapsible below 900px window width — icon-only mode)
- Content padding: 32px
- Card padding: 24px
- Card gap: 16px
- Card border-radius: 12px
- Button border-radius: 8px
- Badge border-radius: 20px (pill)

### 8.4 Window Sizing

- Minimum window size: 800 x 600px
- Default window size: 1200 x 800px
- Below 900px width: sidebar collapses to icon-only (40px wide)
- Charts reflow to single-column layout below 1000px width
- Stat cards stack vertically below 900px width

### 8.5 Interaction

- No heavy shadows — depth via spacing and subtle background shifts
- Hover transitions: 300ms ease
- Number count-up animation on data load (500ms duration, ease-out)
- Charts animate in with drawing motion on first render (800ms)
- Focus-based auto-refresh when window regains attention
- Manual refresh button in top bar with spin animation while syncing
- Chart tooltips on hover showing exact values
- Clickable chart elements to filter related views

### 8.6 Chart Interactions

- **Hover:** tooltip with exact values, date, and tool breakdown
- **Click on daily chart bar/point:** filters the session activity feed to that day
- **No zoom/pan in v1** — use the day/week/month toggle instead
- **Legend:** clickable to show/hide individual tools on the chart

## 9. Settings Page

- **Data sources** — show detected paths, file sizes, status (connected / not found / error), last sync timestamp, session count per source. "Re-scan" button to re-detect sources.
- **Cost rates** — editable per-model token prices with separate rates for input, output, cache read, and cache creation tokens. Not pre-filled — user enters their own rates. Subscription plan users (Claude Max, Codex Pro) can leave blank to hide cost estimates.
- **Subagent handling** — toggle: "Roll up subagent tokens into parent session" (default on) vs "Show subagent threads separately"
- **Export** — export all data as CSV or JSON
- **Diagnostics** — app version, database file size, total sessions/requests count, last sync details. "Copy diagnostics" button for bug reports.
- **About** — version, GitHub link, open source license (MIT)

## 10. Update / Refresh Behavior

### Trigger Conditions
1. **On app launch** — full data sync from both sources (show cached data immediately, sync in background)
2. **On window focus** — incremental refresh (only new data since last sync)
3. **Manual refresh button** — same as incremental refresh
4. **No background process** — app only reads data when it's open/focused

### Incremental Sync Mechanism

**Codex sync watermarks:**
- Store `MAX(updated_at)` from last-read threads in `sync_state`
- Query: `SELECT * FROM threads WHERE updated_at > :last_sync_at`
- For JSONL files: track a set of already-processed rollout file paths. On sync, scan `~/.codex/sessions/` for new files not in the set.

**Claude Code sync watermarks:**
- Store a JSON map of `{filepath: byte_offset}` in `sync_state`
- On sync, check each JSONL file's current size vs stored offset
- Only read new bytes from the end of each file
- Discover new session files by scanning `~/.claude/projects/*/` for files not yet in the map

**Sync pipeline:**
1. Show stale cached data in the UI immediately
2. In a background Rust thread:
   a. Read new entries from Codex SQLite (read-only mode) + new JSONL files
   b. Read new entries from Claude Code JSONL files
   c. Normalize and `INSERT OR IGNORE` into Tally's unified SQLite
   d. Update `sync_state` watermarks
3. Send updated data to React frontend via Tauri event
4. Frontend merges new data and re-renders affected components

### Concurrent SQLite Access

- Open Codex's `state_*.sqlite` in **read-only mode** (`SQLITE_OPEN_READONLY`)
- On Windows, use `?mode=ro&immutable=1` connection string to avoid locking conflicts
- If the database is locked (Codex is actively writing), log a warning and retry on next sync cycle — do not block the UI

### Error Handling

| Scenario | Behavior |
|----------|----------|
| Codex SQLite locked | Skip sync, show stale data, retry on next trigger |
| JSONL file partially written (invalid last line) | Parse all complete lines, skip the incomplete last line |
| Source directory not found | Mark source as "not detected" in Settings, skip silently |
| Tally database corrupted | Offer to reset database in Settings (re-imports from sources) |
| Codex DB version changed (e.g. state_6.sqlite appears) | Auto-discover and switch to new version, re-sync |
| Unknown columns in source tables | Ignore unknown columns, query only known fields |

## 11. Installation & Distribution

### GitHub Releases
- Pre-built `.dmg` for macOS (universal binary: Intel + Apple Silicon)
- Pre-built `.msi` for Windows
- Built via GitHub Actions CI/CD

### From Source
```bash
git clone https://github.com/<user>/tally.git
cd tally
npm install
cargo tauri build
```

### Requirements
- No runtime dependencies for end users
- Node.js + Rust toolchain only needed for building from source

## 12. Security & Privacy

- **Data minimization:** Tally only extracts token counts, model names, timestamps, and project metadata from source files. Conversation content, user prompts, and code are never read, stored, or cached.
- **Local only:** No network requests, no telemetry, no analytics, no cloud services.
- **File permissions:** On launch, verify that source directories (`~/.codex/`, `~/.claude/`) are owned by the current user. Log a warning if permissions seem wrong (shared machine scenario).
- **No shell modification:** Tally never modifies shell profiles, environment variables, or system configuration.
- **Read-only access:** Tally never writes to Codex or Claude Code data directories. It only reads.

## 13. Logging & Diagnostics

- App logs written to `~/.tally/tally.log` (rotated, max 10MB)
- Log levels: error, warn, info (default: info)
- Logs include: sync timestamps, session counts imported, errors encountered, file paths scanned
- Logs never include: conversation content, token values, user prompts
- Settings page "Copy diagnostics" button copies: app version, OS, database size, source paths, last sync status, recent errors

## 14. Non-Goals (v1)

- No cloud sync or accounts
- No mobile app
- No Linux support (can be added later since Tauri supports it)
- No dark mode
- No browser extension
- No tracking of other AI tools (Cursor, Copilot, etc.) — can be added later via plugin architecture
- No notification system
- No pre-filled cost rates (user enters their own)
- No chart zoom/pan (use time range toggles instead)
- No automatic data pruning (manual export + delete via Settings)

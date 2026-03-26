# Activity Calendar Heatmap Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the weekday aggregate card with a month calendar heatmap that uses daily token totals, supports month navigation, and filters the session feed on day click.

**Architecture:** Keep the backend unchanged and move the new month navigation behavior into the React layer. The calendar component will fetch month-specific `dailyUsage` data via the existing Tauri API, derive the display grid and monthly summary client-side, and report selected dates back up to the dashboard for session feed filtering.

**Tech Stack:** React 19, TypeScript, Tauri invoke API, Tailwind CSS v4

---

### Task 1: Add date helpers for month calendar rendering

**Files:**
- Modify: `src/lib/format.ts`

**Step 1: Add month helper functions**

Add small pure helpers for:

- month key formatting
- month label formatting
- start of month
- end of month
- calendar grid start/end calculations
- date comparison in `YYYY-MM-DD` form

**Step 2: Keep helpers ASCII and framework-free**

Do not add dependencies. Keep the functions reusable by both calendar rendering and future dashboard features.

**Step 3: Verify imports remain stable**

Update any call sites added later to use these helpers instead of duplicating date logic.

### Task 2: Rebuild the activity card as a month calendar heatmap

**Files:**
- Modify: `src/components/dashboard/Heatmap.tsx`
- Modify: `src/lib/tauri.ts` only if the component needs a new helper wrapper signature

**Step 1: Replace weekday aggregation**

Remove the current 7-cell weekday summary logic and replace it with:

- visible month state
- async fetch for month `dailyUsage`
- derived calendar cells for the full displayed grid

**Step 2: Implement month navigation**

Add previous/next controls in the card header and refetch month data when the visible month changes.

**Step 3: Implement token intensity rendering**

Shade day cells by `total_tokens` using a per-month max for scaling. Keep inactive days neutral and future days visually subdued.

**Step 4: Implement hover and selection**

Hover should show date, token count, session count, and cost. Clicking a cell with a real date should notify the parent of the selected date.

**Step 5: Add monthly summary + legend**

Render:

- total month tokens
- active days
- most active weekday
- compact intensity legend

### Task 3: Wire selection into the existing dashboard flow

**Files:**
- Modify: `src/pages/Home.tsx`

**Step 1: Pass month anchor input**

Pass the current dashboard range end date into the heatmap so it initializes on the most relevant month.

**Step 2: Pass day selection callback**

Keep using `selectedDate` in `DashboardGrid`, but allow the heatmap to update it via click.

**Step 3: Preserve existing chart behavior**

Do not break the current daily usage chart click-to-filter interaction. The selected date should continue to drive `SessionFeed`.

### Task 4: Verify build and behavior

**Files:**
- No code changes required unless fixes are needed

**Step 1: Run frontend build**

Run: `npm run build`

Expected: TypeScript and Vite build complete successfully.

**Step 2: Sanity check behavior**

Verify:

- month navigation changes visible month
- tooltip reflects token-based stats
- clicking a day updates the session feed filter
- default month follows the current dashboard range end date

### Task 5: Commit implementation

**Files:**
- Commit all files changed for the feature

**Step 1: Stage feature files**

Run: `git add docs/plans/2026-03-25-activity-calendar-design.md docs/plans/2026-03-25-activity-calendar-implementation.md src/lib/format.ts src/components/dashboard/Heatmap.tsx src/pages/Home.tsx`

**Step 2: Commit**

Run: `git commit -m "feat: replace weekday heatmap with monthly activity calendar"`

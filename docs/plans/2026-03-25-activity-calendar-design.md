# Activity Calendar Design

**Date:** 2026-03-25

## Goal

Replace the current `Activity by Day` weekday aggregate with a month-based calendar heatmap that is easier to scan for intra-month token usage patterns while still supporting drill-down into a specific date.

## Current Problem

The existing card is labeled `Activity by Day`, but it actually aggregates the last 30 days into seven weekday buckets. That makes it hard to answer date-oriented questions and does not match the user's preferred calendar mental model.

## Approved Direction

Build a desktop-first monthly calendar heatmap with month navigation.

- Header includes previous and next month buttons plus the visible month label.
- The body renders a standard 7-column calendar grid.
- Each date cell is shaded by total tokens for that day.
- Color intensity is scaled within the visible month, not globally.
- Hovering a populated day shows exact stats.
- Clicking a day selects it and filters the session feed below.

## Data Strategy

Do not add a new backend endpoint for this feature.

The frontend can reuse the existing `getDailyUsage` Tauri command because it already returns per-day totals including:

- `total_tokens`
- `session_count`
- `estimated_cost`

The calendar component will fetch the visible month directly so it can navigate months independently of the broader dashboard date range.

## Interaction Model

### Default state

- The component opens to the month containing the current dashboard range end date when available.
- If no dashboard range exists, it opens to the current month.

### Hover

- Show date
- Show total tokens
- Show sessions
- Show estimated cost when non-zero

### Click

- Select the date
- Highlight the selected cell
- Filter the existing session feed to that date

### Summary row

Show a lightweight monthly summary beneath the calendar:

- Total month tokens
- Active days in the month
- Most active weekday by tokens

## Visual Notes

- Keep the cells compact and uniform so the grid still reads like a pattern view.
- Zero-activity days should remain visible with a neutral fill.
- Today should receive a subtle outline.
- The selected day should receive a stronger ring/border than today.
- Include weekday labels to improve scan speed.
- Include a simple intensity legend to clarify the encoding.

## Edge Cases

- Empty month: render the full grid with neutral cells and an empty-state summary.
- Future dates within the visible month: render as disabled or neutral, with no tooltip stats.
- Months with no selected day: the session feed remains driven by other existing interactions.
- If a selected day is outside the currently visible month, retain the selected date state but only highlight it when its month is shown.

## Testing / Verification

- Build the frontend successfully.
- Verify month navigation updates the grid and summary.
- Verify per-month intensity rescaling by switching between a sparse month and a dense month.
- Verify hover tooltip content for active and inactive days.
- Verify clicking a day updates the session feed filter.
- Verify today and selected-day visual states do not conflict.

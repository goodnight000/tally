/** Format a number with abbreviated suffixes (1.2M, 45.3K, etc.) */
export function formatTokens(n: number): string {
  if (n >= 1_000_000_000) return `${(n / 1_000_000_000).toFixed(1)}B`;
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return n.toLocaleString();
}

/** Format a dollar cost */
export function formatCost(dollars: number): string {
  if (dollars === 0) return "$0.00";
  if (dollars < 0.01) return "<$0.01";
  if (dollars < 1) return `$${dollars.toFixed(2)}`;
  if (dollars < 1000) return `$${dollars.toFixed(2)}`;
  return `$${dollars.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

/** Format a number with full comma-separated value */
export function formatNumber(n: number): string {
  return n.toLocaleString();
}

/** Format bytes to human readable */
export function formatBytes(bytes: number): string {
  if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(1)} GB`;
  if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`;
  if (bytes >= 1_024) return `${(bytes / 1_024).toFixed(1)} KB`;
  return `${bytes} B`;
}

/** Format an ISO date string to relative time */
export function formatRelativeTime(isoString: string): string {
  const date = new Date(isoString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return "just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
}

/** Format ISO date to short date string */
export function formatDate(isoString: string): string {
  return new Date(isoString).toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
  });
}

/** Format ISO date to full date string */
export function formatFullDate(isoString: string): string {
  return new Date(isoString).toLocaleDateString("en-US", {
    weekday: "short",
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

/** Format ISO date to time string */
export function formatTime(isoString: string): string {
  return new Date(isoString).toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
  });
}

/** Calculate estimated cost from tokens and rate */
export function calculateCost(
  tokens: number,
  ratePerMillion: number | null
): number | null {
  if (ratePerMillion == null) return null;
  return (tokens / 1_000_000) * ratePerMillion;
}

/** Get date string N days ago in YYYY-MM-DD format */
export function daysAgo(n: number): string {
  const d = new Date();
  d.setDate(d.getDate() - n);
  return d.toISOString().split("T")[0];
}

/** Get today's date in YYYY-MM-DD format */
export function today(): string {
  return new Date().toISOString().split("T")[0];
}

function parseDateOnly(value: string): Date {
  const datePart = value.slice(0, 10);
  const [year, month, day] = datePart.split("-").map(Number);
  return new Date(year, month - 1, day);
}

function formatDateOnly(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function getMonthKey(value: string): string {
  return value.slice(0, 7);
}

export function shiftMonth(monthKey: string, delta: number): string {
  const [year, month] = monthKey.split("-").map(Number);
  const next = new Date(year, month - 1 + delta, 1);
  return formatDateOnly(next).slice(0, 7);
}

export function formatMonthLabel(monthKey: string): string {
  const [year, month] = monthKey.split("-").map(Number);
  return new Date(year, month - 1, 1).toLocaleDateString("en-US", {
    month: "long",
    year: "numeric",
  });
}

export function getMonthBounds(monthKey: string): { start: string; end: string } {
  const [year, month] = monthKey.split("-").map(Number);
  const start = new Date(year, month - 1, 1);
  const end = new Date(year, month, 0);
  return {
    start: formatDateOnly(start),
    end: formatDateOnly(end),
  };
}

export function getCalendarBounds(monthKey: string): { start: string; end: string } {
  const monthBounds = getMonthBounds(monthKey);
  const start = parseDateOnly(monthBounds.start);
  start.setDate(start.getDate() - start.getDay());

  const end = parseDateOnly(monthBounds.end);
  end.setDate(end.getDate() + (6 - end.getDay()));

  return {
    start: formatDateOnly(start),
    end: formatDateOnly(end),
  };
}

export function getDateRange(start: string, end: string): string[] {
  const results: string[] = [];
  const cursor = parseDateOnly(start);
  const last = parseDateOnly(end);

  while (cursor <= last) {
    results.push(formatDateOnly(cursor));
    cursor.setDate(cursor.getDate() + 1);
  }

  return results;
}

export function isSameDay(a?: string | null, b?: string | null): boolean {
  if (!a || !b) return false;
  return a.slice(0, 10) === b.slice(0, 10);
}

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

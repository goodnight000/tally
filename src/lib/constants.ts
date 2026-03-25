export const COLORS = {
  cream: "#FFFFEB",
  terracotta: "#CC785C",
  periwinkle: "#7B8CEA",
  border: "#E8E8E0",
  textPrimary: "#1A1A1A",
  textSecondary: "#8D8D83",
  interactive: "#4D65FF",
  success: "#2D9E73",
  warning: "#D97706",
  white: "#FFFFFF",
  aggregate: "#7EA07E",
} as const;

export const TOOL_COLORS: Record<string, string> = {
  claude: COLORS.terracotta,
  codex: COLORS.periwinkle,
  cline: "#F59E0B",
  kilo: "#10B981",
  roo: "#8B5CF6",
  opencode: "#06B6D4",
  openclaw: "#EF4444",
  aggregate: COLORS.aggregate,
};

/** Get the chart color for a tool filter (undefined = aggregate/home) */
export function getToolColor(toolFilter?: string): string {
  if (toolFilter && toolFilter in TOOL_COLORS) return TOOL_COLORS[toolFilter];
  return COLORS.aggregate;
}

export const DATE_PRESETS = [
  { label: "Today", value: "today", days: 0 },
  { label: "7 days", value: "7d", days: 7 },
  { label: "30 days", value: "30d", days: 30 },
  { label: "All time", value: "all", days: -1 },
] as const;

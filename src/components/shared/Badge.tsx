interface Props {
  tool?: "claude" | "codex";
  label: string;
  className?: string;
}

export function Badge({ tool, label, className = "" }: Props) {
  const bgColor = tool
    ? tool === "claude"
      ? "bg-terracotta/10 text-terracotta"
      : "bg-periwinkle/10 text-periwinkle"
    : "bg-border text-text-secondary";

  return (
    <span
      className={`inline-flex items-center px-2.5 py-0.5 rounded-(--radius-badge) text-[13px] font-semibold ${bgColor} ${className}`}
    >
      {label}
    </span>
  );
}

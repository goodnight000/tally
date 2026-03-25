import { ReactNode } from "react";

interface Props {
  label: string;
  children: ReactNode;
  subtitle?: string;
  className?: string;
}

export function StatCard({ label, children, subtitle, className = "" }: Props) {
  return (
    <div
      className={`bg-white rounded-(--radius-card) p-(--spacing-card-padding) border border-border ${className}`}
    >
      <p className="text-xs text-text-secondary font-sans mb-1">{label}</p>
      <div className="font-serif text-4xl italic text-text-primary">{children}</div>
      {subtitle && (
        <p className="text-xs text-text-secondary mt-1">{subtitle}</p>
      )}
    </div>
  );
}

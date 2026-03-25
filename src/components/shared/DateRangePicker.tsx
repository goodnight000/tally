import { useState } from "react";
import { DATE_PRESETS } from "../../lib/constants";
import { daysAgo, today } from "../../lib/format";

interface Props {
  onChange: (range: { start: string; end: string } | null) => void;
  className?: string;
}

export function DateRangePicker({ onChange, className = "" }: Props) {
  const [active, setActive] = useState("30d");

  const handlePreset = (value: string, days: number) => {
    setActive(value);
    if (days < 0) {
      onChange(null); // All time
    } else if (days === 0) {
      const t = today();
      onChange({ start: t, end: t + "T23:59:59Z" });
    } else {
      onChange({ start: daysAgo(days), end: today() + "T23:59:59Z" });
    }
  };

  return (
    <div className={`flex gap-1 ${className}`}>
      {DATE_PRESETS.map((preset) => (
        <button
          key={preset.value}
          onClick={() => handlePreset(preset.value, preset.days)}
          className={`px-3 py-1 rounded-(--radius-button) text-xs transition-all duration-300 ${
            active === preset.value
              ? "bg-text-primary text-white"
              : "bg-white text-text-secondary hover:bg-cream border border-border"
          }`}
        >
          {preset.label}
        </button>
      ))}
    </div>
  );
}

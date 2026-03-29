import { describe, it, expect } from "vitest";
import {
  formatTokens,
  formatCost,
  formatBytes,
  calculateCost,
  getMonthBounds,
  getDateRange,
  shiftMonth,
  isSameDay,
} from "./format";

describe("formatTokens", () => {
  it("formats billions", () => {
    expect(formatTokens(1_500_000_000)).toBe("1.5B");
  });

  it("formats millions", () => {
    expect(formatTokens(2_345_678)).toBe("2.3M");
  });

  it("formats thousands", () => {
    expect(formatTokens(45_300)).toBe("45.3K");
  });

  it("formats small numbers with locale string", () => {
    expect(formatTokens(999)).toBe("999");
  });
});

describe("formatCost", () => {
  it("returns $0.00 for zero", () => {
    expect(formatCost(0)).toBe("$0.00");
  });

  it("returns <$0.01 for sub-cent amounts", () => {
    expect(formatCost(0.005)).toBe("<$0.01");
  });

  it("formats cents correctly", () => {
    expect(formatCost(0.42)).toBe("$0.42");
  });

  it("formats dollars with two decimals", () => {
    expect(formatCost(12.5)).toBe("$12.50");
  });

  it("formats large amounts with commas", () => {
    const result = formatCost(1234.56);
    expect(result).toContain("1,234.56") ;
  });
});

describe("formatBytes", () => {
  it("formats gigabytes", () => {
    expect(formatBytes(2_147_483_648)).toBe("2.0 GB");
  });

  it("formats megabytes", () => {
    expect(formatBytes(5_242_880)).toBe("5.0 MB");
  });

  it("formats kilobytes", () => {
    expect(formatBytes(2_048)).toBe("2.0 KB");
  });

  it("formats bytes", () => {
    expect(formatBytes(512)).toBe("512 B");
  });
});

describe("calculateCost", () => {
  it("returns null when rate is null", () => {
    expect(calculateCost(1000, null)).toBeNull();
  });

  it("calculates cost correctly", () => {
    expect(calculateCost(1_000_000, 3.0)).toBe(3.0);
  });

  it("handles zero tokens", () => {
    expect(calculateCost(0, 15.0)).toBe(0);
  });
});

describe("getMonthBounds", () => {
  it("returns first and last day of a month", () => {
    const bounds = getMonthBounds("2026-03");
    expect(bounds.start).toBe("2026-03-01");
    expect(bounds.end).toBe("2026-03-31");
  });

  it("handles February in a non-leap year", () => {
    const bounds = getMonthBounds("2025-02");
    expect(bounds.start).toBe("2025-02-01");
    expect(bounds.end).toBe("2025-02-28");
  });
});

describe("shiftMonth", () => {
  it("shifts forward", () => {
    expect(shiftMonth("2026-03", 1)).toBe("2026-04");
  });

  it("shifts backward across year boundary", () => {
    expect(shiftMonth("2026-01", -1)).toBe("2025-12");
  });
});

describe("getDateRange", () => {
  it("returns inclusive range of dates", () => {
    const range = getDateRange("2026-03-28", "2026-03-31");
    expect(range).toEqual(["2026-03-28", "2026-03-29", "2026-03-30", "2026-03-31"]);
  });

  it("returns single date when start equals end", () => {
    const range = getDateRange("2026-03-29", "2026-03-29");
    expect(range).toEqual(["2026-03-29"]);
  });
});

describe("isSameDay", () => {
  it("returns true for same date", () => {
    expect(isSameDay("2026-03-29T10:00:00Z", "2026-03-29T23:59:59Z")).toBe(true);
  });

  it("returns false for different dates", () => {
    expect(isSameDay("2026-03-28", "2026-03-29")).toBe(false);
  });

  it("returns false for null values", () => {
    expect(isSameDay(null, "2026-03-29")).toBe(false);
    expect(isSameDay(undefined, undefined)).toBe(false);
  });
});

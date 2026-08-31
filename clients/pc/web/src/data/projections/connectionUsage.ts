import type { UsageEventView } from "./providers";

/**
 * Settings connection-table usage cell (P12-T08). Unknown and
 * cost_unavailable stay text. They are never rendered as 0.
 */
export function connectionUsageLabel(accountId: string, events: UsageEventView[]): string {
  const mine = events.filter((event) => event.accountId === accountId);
  if (mine.length === 0) {
    return "unknown";
  }
  if (mine.some((event) => event.costStatus === "cost_unavailable")) {
    return "cost_unavailable";
  }
  if (mine.some((event) => event.costMicros == null || event.costStatus === "unknown")) {
    return "unknown";
  }
  const sum = mine.reduce((total, event) => total + (event.costMicros ?? 0), 0);
  return `$${(sum / 1_000_000).toFixed(6)}`;
}

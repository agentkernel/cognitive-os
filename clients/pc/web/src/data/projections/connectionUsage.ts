import type { UsageEventView } from "./providers";

/**
 * Settings connection-table usage cell (P12-T08 / P13-T08).
 * Source-labelled actual / estimated / unknown. Unknown and
 * cost_unavailable stay text. They are never rendered as 0.
 */
export function connectionUsageLabel(accountId: string, events: UsageEventView[]): string {
  const mine = events.filter((event) => event.accountId === accountId);
  if (mine.length === 0) {
    return "unknown";
  }
  if (
    mine.some(
      (event) =>
        event.costLabel === "unknown" ||
        event.costStatus === "cost_unavailable" ||
        event.costStatus === "unknown" ||
        event.costMicros == null,
    )
  ) {
    return "unknown";
  }
  const estimated = mine.some((event) => event.costLabel === "estimated");
  const sum = mine.reduce((total, event) => total + (event.costMicros ?? 0), 0);
  const source = estimated ? "estimated" : "actual";
  return `${source} $${(sum / 1_000_000).toFixed(6)}`;
}

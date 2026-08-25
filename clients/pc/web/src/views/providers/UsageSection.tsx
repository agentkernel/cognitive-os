import { HonestyNote } from "../../state/HonestyNote";
import type {
  BudgetView,
  UsageEventView,
} from "../../data/projections/providers";
import { usageCostLabel } from "../../data/projections/providers";
import type { Projection } from "../../data/store";
import { ProjectionState } from "./ProjectionState";

/**
 * Usage section — docs/design/17 §2.4. Token/cost counters with unknown
 * rendered as unknown (never 0); cost_unavailable events are counted
 * separately, never summed as $0. Budgets are observe-only and labeled
 * advisory — they never block (BD-8). No charts: honest counters beat
 * decorative graphs.
 */
export function UsageSection({
  accountId,
  usage,
  budgets,
}: {
  accountId: string;
  usage: Projection<UsageEventView[]>;
  budgets: Projection<BudgetView[]>;
}) {
  const events = (usage.data ?? []).filter((event) => event.accountId === accountId);
  const priced = events.filter(
    (event) => event.costStatus === "priced" && event.costMicros != null,
  );
  const unavailable = events.filter(
    (event) => event.costStatus === "cost_unavailable" || event.costMicros == null,
  );
  const totalMicros = priced.reduce((sum, event) => sum + (event.costMicros ?? 0), 0);
  const accountBudgets = (budgets.data ?? []).filter(
    (budget) => budget.scopeId === accountId,
  );

  return (
    <>
      <ProjectionState projection={usage} what="Usage" />
      {usage.status === "ready" || usage.status === "empty" || usage.status === "stale" ? (
        <>
          <HonestyNote>
            Unknown is not zero. Events with <code className="cp-mono">cost_unavailable</code>{" "}
            are counted separately and never summed as $0.
          </HonestyNote>
          <p className="cp-quiet">
            {events.length} events · {priced.length} priced · {unavailable.length}{" "}
            cost-unavailable
            {priced.length > 0
              ? ` · priced total $${(totalMicros / 1_000_000).toFixed(6)}`
              : " · priced total unknown"}
          </p>
          {events.length > 0 ? (
            <table className="cp-table">
              <caption>Usage events for this account (bounded to 50)</caption>
              <thead>
                <tr>
                  <th scope="col">Event</th>
                  <th scope="col">Cost</th>
                </tr>
              </thead>
              <tbody>
                {events.slice(0, 50).map((event) => (
                  <tr key={event.id}>
                    <td className="cp-mono">{event.id}</td>
                    <td className="cp-mono">{usageCostLabel(event)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <p className="cp-quiet">No usage events recorded for this account.</p>
          )}
          <ProjectionState projection={budgets} what="Budgets" />
          {budgets.status === "ready" ||
          budgets.status === "empty" ||
          budgets.status === "stale" ? (
            <>
              <h4 className="cp-section-title">Budgets</h4>
              <HonestyNote>
                Budgets are observe-only; they never block dispatch (BD-8).
              </HonestyNote>
              {accountBudgets.length === 0 ? (
                <p className="cp-quiet">No budgets scoped to this account.</p>
              ) : (
                <table className="cp-table">
                  <caption>Budgets scoped to this account (advisory)</caption>
                  <thead>
                    <tr>
                      <th scope="col">Budget</th>
                      <th scope="col">Scope</th>
                      <th scope="col">Token limit</th>
                      <th scope="col">Amount limit</th>
                    </tr>
                  </thead>
                  <tbody>
                    {accountBudgets.map((budget) => (
                      <tr key={budget.id}>
                        <td className="cp-mono">{budget.id}</td>
                        <td className="cp-mono">
                          {budget.scopeKind}:{budget.scopeId}
                        </td>
                        <td className="cp-mono">{budget.tokenLimit ?? "unknown"}</td>
                        <td className="cp-mono">
                          {budget.amountMicrosLimit == null
                            ? "unknown"
                            : `$${(budget.amountMicrosLimit / 1_000_000).toFixed(2)}`}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </>
          ) : null}
        </>
      ) : null}
    </>
  );
}

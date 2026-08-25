import { useState } from "react";
import { readJson } from "../../api";
import { ReceiptLine } from "../../components/ReceiptLine";
import { asRecord } from "../../data/projections";
import type {
  BudgetView,
  ProviderAlertView,
} from "../../data/projections/providers";
import type { Projection } from "../../data/store";
import { HonestyNote } from "../../state/HonestyNote";
import { ProjectionState } from "./ProjectionState";

/**
 * Alerts block — docs/design/17 §2.4. Alerts link to this account through
 * their budget's scope (the only linkage the daemon exposes). Acknowledge
 * is a class-B action with a persistent receipt or an explicit error state.
 * Alerts are advisory and never block.
 */
export function AlertsBlock({
  accountId,
  alerts,
  budgets,
  onChanged,
}: {
  accountId: string;
  alerts: Projection<ProviderAlertView[]>;
  budgets: Projection<BudgetView[]>;
  onChanged: () => void;
}) {
  const [message, setMessage] = useState<string | undefined>();
  const [receipt, setReceipt] = useState<string | undefined>();

  const scopedBudgetIds = new Set(
    (budgets.data ?? [])
      .filter((budget) => budget.scopeId === accountId)
      .map((budget) => budget.id),
  );
  const rows = (alerts.data ?? []).filter(
    (alert) => alert.budgetId != null && scopedBudgetIds.has(alert.budgetId),
  );

  async function acknowledge(id: string) {
    setReceipt(undefined);
    setMessage(undefined);
    const result = await readJson("/management/alerts/acknowledge", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ alert_id: id }),
    });
    if (result.ok) {
      setReceipt(`Alert ${id} acknowledged.`);
    } else {
      setMessage(`HTTP ${result.status} ${String(asRecord(result.body).code ?? "")}`);
    }
    onChanged();
  }

  return (
    <div className="cp-subblock">
      <h4 className="cp-section-title">Alerts</h4>
      <HonestyNote>
        Alerts are advisory and never block. They link to this account through its budgets —
        the daemon exposes no direct account linkage.
      </HonestyNote>
      <ProjectionState projection={alerts} what="Alerts" />
      {alerts.status === "ready" || alerts.status === "empty" || alerts.status === "stale" ? (
        <>
          {rows.length === 0 ? (
            <p className="cp-quiet">No alerts linked to this account&apos;s budgets.</p>
          ) : (
            <table className="cp-table">
              <caption>Alerts on this account&apos;s budgets (advisory)</caption>
              <thead>
                <tr>
                  <th scope="col">Alert</th>
                  <th scope="col">Threshold</th>
                  <th scope="col">State</th>
                  <th scope="col" aria-label="actions" />
                </tr>
              </thead>
              <tbody>
                {rows.map((alert) => (
                  <tr key={alert.id}>
                    <td className="cp-mono">{alert.id}</td>
                    <td className="cp-mono">{alert.threshold}</td>
                    <td>{alert.acknowledged ? "acknowledged" : "unacknowledged"}</td>
                    <td>
                      {alert.acknowledged ? null : (
                        <button
                          type="button"
                          className="cp-button"
                          onClick={() => void acknowledge(alert.id)}
                        >
                          Acknowledge
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </>
      ) : null}
      {/*
       * Acknowledge triggers its own refresh; the receipt and the error must
       * outlive that refresh, so they render outside the projection guard.
       * A receipt is a record of an authority act, not ephemera.
       */}
      {receipt ? <ReceiptLine>{receipt}</ReceiptLine> : null}
      {message ? (
        <p role="alert" className="cp-reason">
          {message}
        </p>
      ) : null}
    </div>
  );
}

import { useState } from "react";
import { Link } from "react-router-dom";
import { readJson } from "../../api";
import { ReceiptLine } from "../../components/ReceiptLine";
import { asRecord } from "../../data/projections";
import {
  ATTENTION_ROW_CAP,
  formatAge,
  type AttentionItem,
} from "../../data/projections/home";
import type { Projection } from "../../data/store";
import type { LastGood } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { StateChip } from "../../state/StateChip";
import { RegionStatus } from "./RegionStatus";

export interface AttentionSource {
  what: string;
  projection: Projection<unknown>;
  lastGood: LastGood<unknown>;
}

/**
 * R2 Needs attention — docs/design/13 §3. The one queue: consequential
 * changes on top, then blocked/failed, attention, waiting-on-owner, stale.
 * Every row is a navigable authority fact with exactly one next action.
 *
 * This is not a notification centre: an alert appears here once, with its
 * class-B acknowledge inline, and the receipt or error outlives the refresh
 * the acknowledge itself triggers.
 */
export function AttentionSection({
  items,
  sources,
  nowMs,
  lastCheckedLabel,
  onAcknowledged,
}: {
  items: AttentionItem[];
  sources: AttentionSource[];
  nowMs: number;
  lastCheckedLabel?: string;
  onAcknowledged: (alertId: string) => void;
}) {
  const [expanded, setExpanded] = useState(false);
  const [receipt, setReceipt] = useState<string | undefined>();
  const [message, setMessage] = useState<string | undefined>();

  const visible = expanded ? items : items.slice(0, ATTENTION_ROW_CAP);
  const hidden = items.length - visible.length;
  const changes = visible.filter((item) => item.priority === "change");
  const rest = visible.filter((item) => item.priority !== "change");

  async function acknowledge(alertId: string) {
    setReceipt(undefined);
    setMessage(undefined);
    const result = await readJson("/management/alerts/acknowledge", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ alert_id: alertId }),
    });
    if (result.ok) {
      setReceipt(`Alert ${alertId} acknowledged.`);
    } else {
      setMessage(`HTTP ${result.status} ${String(asRecord(result.body).code ?? "")}`);
    }
    onAcknowledged(alertId);
  }

  function renderRow(item: AttentionItem) {
    const age =
      formatAge(item.atMs, nowMs) ??
      `age unknown${item.ageUnknownReason ? ` (${item.ageUnknownReason})` : ""}`;
    return (
      <li className="cp-queue-row" key={item.id} data-priority={item.priority}>
        <span className="cp-queue-state">
          <StateChip reading={item.reading} />
        </span>
        <span className="cp-queue-object">
          <span className="cp-quiet">{item.objectType}</span>{" "}
          <code className="cp-mono" title={item.objectRef ?? item.objectLabel}>
            {item.objectLabel}
          </code>
        </span>
        <span className="cp-queue-reason">{item.reason}</span>
        <span className="cp-quiet cp-queue-age">{age}</span>
        <span className="cp-queue-action">
          {item.action?.kind === "link" && item.action.to ? (
            <Link to={item.action.to}>{item.action.label}</Link>
          ) : null}
          {item.action?.kind === "acknowledge" && item.action.alertId ? (
            <button
              type="button"
              className="cp-button"
              onClick={() => void acknowledge(item.action?.alertId ?? "")}
            >
              {item.action.label}
            </button>
          ) : null}
        </span>
      </li>
    );
  }

  return (
    <section className="cp-region" aria-labelledby="home-attention-title">
      <h3 className="cp-section-title" id="home-attention-title">
        Needs attention
      </h3>
      {sources.map((source) => (
        <RegionStatus
          key={source.what}
          projection={source.projection}
          lastGood={source.lastGood}
          what={source.what}
        />
      ))}
      {changes.length > 0 ? (
        <div className="cp-queue-group">
          <h4 className="cp-section-title" id="home-attention-changes">
            Critical changes
          </h4>
          <HonestyNote>
            Coverage is the provider control plane plus this session&apos;s own mutation
            receipts. This is <strong>not</strong> a unified system-wide audit (BD-5): memory,
            skill, tool-lifecycle, backup/restore and task mutations are not audited over HTTP,
            and provider audit rows carry no timestamp, so their age is unknown.
          </HonestyNote>
          <ul className="cp-queue" aria-labelledby="home-attention-changes">
            {changes.map(renderRow)}
          </ul>
        </div>
      ) : null}
      {rest.length > 0 ? (
        <ul className="cp-queue" aria-labelledby="home-attention-title">
          {rest.map(renderRow)}
        </ul>
      ) : null}
      {items.length === 0 ? (
        <p className="cp-region-line">
          Ready. Nothing needs you.{" "}
          <span className="cp-quiet">
            {lastCheckedLabel
              ? `Readiness last checked ${lastCheckedLabel}.`
              : "Readiness last-checked time is unknown."}
          </span>
        </p>
      ) : null}
      {hidden > 0 ? (
        <p className="cp-next">
          <button type="button" className="cp-button" onClick={() => setExpanded(true)}>
            {hidden} more
          </button>{" "}
          <Link to="/activity">Open Activity</Link>
        </p>
      ) : null}
      {/*
       * Acknowledge triggers its own refresh of the alerts projection; the
       * receipt and the error must outlive it. A receipt is the record of an
       * authority act, not a toast.
       */}
      {receipt ? <ReceiptLine>{receipt}</ReceiptLine> : null}
      {message ? (
        <p role="alert" className="cp-reason">
          {message}
        </p>
      ) : null}
    </section>
  );
}

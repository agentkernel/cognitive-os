import { useState } from "react";
import { Link } from "react-router-dom";
import { DigestChip } from "../../components/DigestChip";
import { FactGrid } from "../../components/FactGrid";
import { Inspector } from "../../components/Inspector";
import {
  UNSUPPORTED_TASK_OPERATIONS,
  factOrUnknown,
  rollUpEffects,
  workRowAge,
  workRowObjective,
  workRowReading,
  type EffectEntryView,
  type TaskEvidenceView,
  type WorkRow,
} from "../../data/projections/work";
import type { Projection } from "../../data/store";
import { HonestyNote } from "../../state/HonestyNote";
import { StateChip } from "../../state/StateChip";
import { readDomainState } from "../../state/stateMap";
import { detailPath } from "./WorkInventory";

function ProbeLine({ what, projection }: { what: string; projection?: Projection<unknown> }) {
  if (!projection) {
    return (
      <p className="cp-quiet">
        {what} has not been read for this ref yet — that is a gap in what this page asked for, not
        a fact about the task.
      </p>
    );
  }
  if (projection.status === "loading" || projection.status === "stale") {
    return <p className="cp-quiet">Reading {what}…</p>;
  }
  if (projection.data === undefined) {
    return (
      <p className="cp-reason" role="status">
        <StateChip reading={readDomainState("load", projection.status)} /> {what} could not be read
        — <code className="cp-mono">{projection.error?.code ?? projection.status}</code>. This task
        is not shown as verified and is not shown as failed.
      </p>
    );
  }
  return null;
}

/**
 * The 5-minute layer beside the inventory — docs/design/14 §4. Facts the
 * daemon actually returned for one ref, and an explicit account of what it
 * does not expose.
 *
 * Task detail (timeline, watch attach, per-effect drill-down) is the Work
 * detail view in W5. Until that exists this inspector links nowhere: an
 * "Open detail" control pointing at a blank route would be a fake affordance.
 */
export function WorkInspector({
  row,
  evidence,
  effects,
  nowMs,
  listStateSearch,
}: {
  row: WorkRow;
  evidence?: Projection<TaskEvidenceView>;
  effects?: Projection<EffectEntryView[]>;
  nowMs: number;
  listStateSearch: string;
}) {
  const [copied, setCopied] = useState(false);
  const evidenceView = evidence?.data;
  const effectRows = effects?.data;
  const rollup = effectRows ? rollUpEffects(effectRows) : undefined;

  return (
    <Inspector title="Task" label={`Task ${row.shortRef} inspector`}>
      <p className="cp-region-line">
        <StateChip reading={workRowReading(row)} />
      </p>
      <FactGrid
        facts={[
          {
            label: "Task ref",
            value: (
              <span className="cp-mono" title={row.taskRef}>
                {row.taskRef}
              </span>
            ),
          },
          {
            label: "Contract epoch",
            value: <span className="cp-mono">{factOrUnknown(row.contractEpoch)}</span>,
          },
          {
            label: "Contract digest",
            value: row.revisionDigest ? (
              <DigestChip value={row.revisionDigest} label="contract digest" />
            ) : (
              "unknown"
            ),
          },
          { label: "Objective", value: workRowObjective(row) },
          { label: "Agent", value: "unknown — the daemon exposes no task→agent binding" },
          { label: "Source", value: row.origin },
          { label: "Observed", value: workRowAge(row, nowMs) },
          {
            label: "Envelope health",
            value: <span className="cp-mono">{factOrUnknown(row.health)}</span>,
          },
        ]}
      />
      <p className="cp-next">
        <Link
          className="cp-button cp-button--primary"
          to={detailPath(row.taskRef, listStateSearch)}
        >
          Open detail
        </Link>{" "}
        <button
          type="button"
          className="cp-button"
          onClick={() => {
            void navigator.clipboard.writeText(row.taskRef).then(() => {
              setCopied(true);
              setTimeout(() => setCopied(false), 1200);
            });
          }}
        >
          Copy task ref
        </button>{" "}
        {copied ? <span className="cp-quiet">copied</span> : null}
      </p>

      <h4 className="cp-section-title">Evidence</h4>
      <ProbeLine what="verification evidence" projection={evidence} />
      {evidenceView ? (
        <FactGrid
          facts={[
            {
              label: "Lifecycle state",
              value: <span className="cp-mono">{factOrUnknown(evidenceView.lifecycleState)}</span>,
            },
            {
              label: "Verification",
              value: evidenceView.hasVerification
                ? `${factOrUnknown(evidenceView.verificationStatus)}${
                    evidenceView.verificationCurrent === false
                      ? " — not current for the task's fencing epoch, so it does not prove the current state"
                      : ""
                  }`
                : "no verification report — nothing is proven yet",
            },
            {
              label: "Report digest",
              value: evidenceView.reportDigest ? (
                <DigestChip value={evidenceView.reportDigest} label="report digest" />
              ) : (
                "unknown"
              ),
            },
            {
              label: "Terminal acceptance",
              value: evidenceView.acceptancePresent
                ? `recorded${evidenceView.acceptanceCurrent === false ? " (not current)" : ""}`
                : "none recorded",
            },
          ]}
        />
      ) : null}

      <h4 className="cp-section-title">Effects</h4>
      <ProbeLine what="effect history" projection={effects} />
      {rollup ? (
        rollup.total === 0 ? (
          <p className="cp-quiet">
            The daemon holds no Effect for this task ref. That is an authoritative empty, not a
            pending read.
          </p>
        ) : (
          <FactGrid
            facts={[
              { label: "Effects", value: <span className="cp-mono">{rollup.total}</span> },
              {
                label: "By stage",
                value: (
                  <span className="cp-mono">
                    {rollup.byStage.map((entry) => `${entry.stage}×${entry.count}`).join(" · ")}
                  </span>
                ),
              },
              {
                label: "Needing reconciliation",
                value: <span className="cp-mono">{rollup.needsReconcile}</span>,
              },
            ]}
          />
        )
      ) : null}

      <h4 className="cp-section-title">Watch</h4>
      <p className="cp-quiet">
        No watch stream is attached from this space, and none is attached from the detail view
        either — live delivery arrives with W11. The composed run timeline, per-effect detail and
        consumption pins are in the detail view; detaching a watch has never cancelled a task, and
        an unattached watch says nothing about progress.
      </p>

      <h4 className="cp-section-title">Not available</h4>
      <ul className="cp-plain-list">
        {UNSUPPORTED_TASK_OPERATIONS.map((entry) => (
          <li key={entry.operation}>
            <code className="cp-mono">{entry.operation}</code> — not available over HTTP:{" "}
            {entry.reason}.
          </li>
        ))}
      </ul>
      <HonestyNote>
        These are stated rather than rendered as disabled controls: a greyed-out button would
        claim the capability exists and is merely unavailable right now. There is no such route on
        this daemon.
      </HonestyNote>
    </Inspector>
  );
}

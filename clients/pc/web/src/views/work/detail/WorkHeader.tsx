import { useState } from "react";
import { DigestChip } from "../../../components/DigestChip";
import { FactGrid } from "../../../components/FactGrid";
import { UNSUPPORTED_TASK_OPERATIONS, factOrUnknown } from "../../../data/projections/work";
import {
  type CompletionReading,
  type TaskEvidenceDetail,
} from "../../../data/projections/workDetail";
import { HonestyNote } from "../../../state/HonestyNote";
import { StateChip } from "../../../state/StateChip";
import { readDomainState } from "../../../state/stateMap";
import type { WatchSessionSnapshot } from "../../../watchStream";
import { WatchBar } from "./WatchBar";

/**
 * Detail header — docs/design/15 §1. The task's identity and its verbatim
 * state, plus the reason and epoch that qualify it. Nothing here is a control:
 * the daemon has no cancel, pause or retry route, so those are stated as
 * class-C facts rather than drawn as buttons.
 */
export function WorkHeader({
  taskRef,
  evidence,
  completion,
  evidenceReadable,
  watch,
  onAttach,
  onDetach,
  onReconnect,
}: {
  taskRef: string;
  evidence?: TaskEvidenceDetail;
  completion: CompletionReading;
  evidenceReadable: boolean;
  watch: WatchSessionSnapshot;
  onAttach: () => void;
  onDetach: () => void;
  onReconnect: () => void;
}) {
  const [copied, setCopied] = useState(false);
  const lifecycle = evidence?.currentState;
  const latest = evidence?.transitions.reduce<
    { sequence: number; reasonCode?: string } | undefined
  >((newest, transition) => (newest && newest.sequence > transition.sequence ? newest : transition), undefined);

  return (
    <header className="cp-detail-head">
      <p className="cp-region-line">
        <StateChip
          reading={
            lifecycle
              ? readDomainState("task", lifecycle)
              : { category: "unknown", label: "state not exposed", unmapped: false }
          }
        />{" "}
        <span className="cp-quiet">
          {evidenceReadable
            ? "verbatim lifecycle state from /task/evidence"
            : "no lifecycle read succeeded for this ref"}
        </span>
      </p>
      <h2 className="cp-mono cp-detail-title">{taskRef}</h2>
      <FactGrid
        facts={[
          {
            label: "Contract epoch",
            value: <span className="cp-mono">{factOrUnknown(evidence?.contractEpoch)}</span>,
          },
          {
            label: "Task version",
            value: <span className="cp-mono">{factOrUnknown(evidence?.currentVersion)}</span>,
          },
          {
            label: "Latest reason code",
            value: (
              <span className="cp-mono">
                {latest?.reasonCode ?? "none recorded on the latest transition"}
              </span>
            ),
          },
          { label: "Verification / acceptance", value: completion.label },
          {
            label: "Reconcile class",
            value: <span className="cp-mono">{factOrUnknown(evidence?.reconcileClass)}</span>,
          },
        ]}
      />
      <p className="cp-next">
        <button
          type="button"
          className="cp-button"
          onClick={() => {
            void navigator.clipboard.writeText(taskRef).then(() => {
              setCopied(true);
              setTimeout(() => setCopied(false), 1200);
            });
          }}
        >
          Copy task ref
        </button>{" "}
        {copied ? <span className="cp-quiet">copied</span> : null}
      </p>
      <WatchBar
        snapshot={watch}
        onAttach={onAttach}
        onDetach={onDetach}
        onReconnect={onReconnect}
        variant="header"
      />
      {evidence?.acceptance?.terminalTransitionDigest ? (
        <p className="cp-next">
          <DigestChip
            value={evidence.acceptance.terminalTransitionDigest}
            label="terminal transition digest"
          />
        </p>
      ) : null}
      <h3 className="cp-section-title">Not available over HTTP</h3>
      <ul className="cp-plain-list">
        {UNSUPPORTED_TASK_OPERATIONS.map((entry) => (
          <li key={entry.operation}>
            <code className="cp-mono">{entry.operation}</code> — {entry.reason}.
          </li>
        ))}
      </ul>
      <HonestyNote>
        These are stated, not drawn as disabled controls: a greyed-out button would claim the
        capability exists and is merely unavailable right now. The daemon has no such route, so
        there is nothing to enable.
      </HonestyNote>
    </header>
  );
}

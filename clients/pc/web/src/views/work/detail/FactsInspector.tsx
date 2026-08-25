import { DigestChip } from "../../../components/DigestChip";
import { FactGrid } from "../../../components/FactGrid";
import { Inspector } from "../../../components/Inspector";
import type { SessionTaskChain } from "../../../data/projections/work";
import {
  type CompletionReading,
  type EffectHistoryView,
  type TaskEvidenceDetail,
} from "../../../data/projections/workDetail";
import { HonestyNote } from "../../../state/HonestyNote";

/**
 * Facts inspector — docs/design/15 §1.2. The refs and digests an operator
 * copies when they need to check this task against the daemon itself. Every
 * value here came from a real read; absent values say so.
 */
export function FactsInspector({
  taskRef,
  evidence,
  effects,
  completion,
  chain,
}: {
  taskRef: string;
  evidence?: TaskEvidenceDetail;
  effects?: EffectHistoryView;
  completion: CompletionReading;
  chain?: SessionTaskChain;
}) {
  return (
    <Inspector title="Facts" label={`Task facts for ${taskRef}`}>
      <FactGrid
        facts={[
          {
            label: "Task ref",
            value: (
              <span className="cp-mono" title={taskRef}>
                {taskRef}
              </span>
            ),
          },
          {
            label: "Lifecycle state",
            value: (
              <span className="cp-mono">
                {evidence?.currentState ?? "state not exposed"}
              </span>
            ),
          },
          { label: "Disposition", value: completion.label },
          {
            label: "Contract epoch",
            value: (
              <span className="cp-mono">
                {evidence?.contractEpoch == null ? "unknown" : String(evidence.contractEpoch)}
              </span>
            ),
          },
          {
            label: "Transitions returned",
            value: (
              <span className="cp-mono">
                {evidence == null
                  ? "unknown"
                  : `${evidence.transitions.length}${evidence.transitionsTruncated ? " (bounded)" : ""}`}
              </span>
            ),
          },
          {
            label: "Effects returned",
            value: (
              <span className="cp-mono">
                {effects == null
                  ? "unknown"
                  : `${effects.entries.length}${effects.effectsTruncated ? " (bounded)" : ""}`}
              </span>
            ),
          },
          {
            label: "Report digest",
            value: evidence?.verification?.reportDigest ? (
              <DigestChip value={evidence.verification.reportDigest} label="report digest" />
            ) : (
              "none recorded"
            ),
          },
          {
            label: "Interpretation digest",
            value: chain ? (
              <DigestChip
                value={chain.interpretation.interpretationDigest}
                label="interpretation digest"
              />
            ) : (
              "unavailable in this session"
            ),
          },
          {
            label: "Preview digest",
            value: chain ? (
              <DigestChip value={chain.preview.previewDigest} label="preview digest" />
            ) : (
              "unavailable — previews are never persisted"
            ),
          },
        ]}
      />
      <HonestyNote>
        Copy any ref or digest and check it against the daemon directly. Nothing in this panel is
        derived, defaulted or rounded.
      </HonestyNote>
    </Inspector>
  );
}

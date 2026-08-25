import { DigestChip } from "../../components/DigestChip";
import { FactGrid } from "../../components/FactGrid";
import type { WorkspaceSearchDraft } from "../../taskDraft";
import type { PreviewView } from "../../data/projections/work";
import { HonestyNote } from "../../state/HonestyNote";

/**
 * Step 3 — docs/design/14 §5. The digest-bound review. What is shown here is
 * the contract the daemon previewed, and admission is bound to this exact
 * preview digest: the operator accepts the bytes they read, not a description
 * of them.
 */
export function TaskPreview({
  preview,
  draft,
  interpretationId,
  principal,
  onAdmit,
  onCancel,
  busy,
  needsReconfirm,
}: {
  preview: PreviewView;
  draft: WorkspaceSearchDraft;
  interpretationId: string;
  principal: string;
  onAdmit: () => void;
  onCancel: () => void;
  busy: boolean;
  /** True after a 409: the previous digest is void and this one is fresh. */
  needsReconfirm: boolean;
}) {
  const budget = preview.budget ?? draft.budget;
  return (
    <section className="cp-region" aria-labelledby="preview-title">
      <h3 className="cp-section-title" id="preview-title">
        Confirm the contract
      </h3>
      {needsReconfirm ? (
        <p className="cp-reason" role="status">
          This is a freshly generated preview after the daemon refused the previous one. It carries
          a new digest and needs its own explicit confirmation — nothing was retried for you.
        </p>
      ) : null}
      <FactGrid
        facts={[
          { label: "Objective", value: preview.objective || draft.objective },
          {
            label: "Task ref",
            value: <span className="cp-mono">{preview.taskRef || draft.task_ref}</span>,
          },
          {
            label: "Preview digest",
            value: <DigestChip value={preview.previewDigest} label="preview digest" />,
          },
          {
            label: "Interpretation",
            value: <span className="cp-mono">{interpretationId}</span>,
          },
          {
            label: "In scope",
            value: draft.scope.in_scope.join(", ") || "unknown",
          },
          {
            label: "Out of scope",
            value: draft.scope.out_of_scope.join(", ") || "unknown",
          },
          {
            label: "Acceptance conditions",
            value: (
              <ul className="cp-plain-list">
                {draft.conditions.map((condition) => (
                  <li key={condition.id}>
                    {condition.description} —{" "}
                    <code className="cp-mono">{condition.verifier_ref}</code>
                  </li>
                ))}
              </ul>
            ),
          },
          {
            label: "Condition count (daemon)",
            value: (
              <span className="cp-mono">
                {preview.conditionCount == null ? "unknown" : String(preview.conditionCount)}
              </span>
            ),
          },
          {
            label: "Budget",
            value: (
              <span className="cp-mono">
                {Object.entries(budget)
                  .map(([key, value]) => `${key}=${String(value)}`)
                  .join(" · ")}
              </span>
            ),
          },
          { label: "Deadline", value: <span className="cp-mono">{draft.deadline}</span> },
          {
            label: "Allowed tools",
            value: <span className="cp-mono">{draft.allowed_tools.join(", ")}</span>,
          },
          {
            label: "Allowed state domains",
            value: <span className="cp-mono">{draft.allowed_state_domains.join(", ")}</span>,
          },
          {
            label: "Max iterations / retries",
            value: (
              <span className="cp-mono">
                {draft.max_iterations} / {draft.max_retries}
              </span>
            ),
          },
          { label: "Accepted by", value: <span className="cp-mono">{principal}</span> },
        ]}
      />
      <p className="cp-next">
        <button
          type="button"
          className="cp-button cp-button--primary"
          onClick={onAdmit}
          disabled={busy}
        >
          Confirm and admit
        </button>{" "}
        <button type="button" className="cp-button" onClick={onCancel} disabled={busy}>
          Back to the interpretation
        </button>
      </p>
      <HonestyNote>
        Admission binds this preview digest, this interpretation and the principal currently
        authenticated on the Task session. If the daemon rejects the tuple it returns HTTP 409 and
        nothing is admitted — this page will generate a new preview and ask you to confirm again
        rather than retrying silently. A successful admit means the contract was admitted; it does
        not mean the task has run, progressed or completed.
      </HonestyNote>
    </section>
  );
}

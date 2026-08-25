import { DigestChip } from "../../../components/DigestChip";
import { FactGrid } from "../../../components/FactGrid";
import {
  type CompletionReading,
  type TaskEvidenceDetail,
} from "../../../data/projections/workDetail";
import type { Projection } from "../../../data/store";
import { HonestyNote } from "../../../state/HonestyNote";
import { StateChip } from "../../../state/StateChip";
import { readDomainState } from "../../../state/stateMap";

/**
 * Evidence — docs/design/15 §3.4. The only section allowed to produce the word
 * "completed", and only when a current terminal acceptance record exists. A
 * passing verification report is not completion, and a report that is not
 * current for the fencing epoch proves nothing about the current state.
 */
export function EvidenceSection({
  evidence,
  completion,
  projection,
}: {
  evidence?: TaskEvidenceDetail;
  completion: CompletionReading;
  projection?: Projection<unknown>;
}) {
  const notFound = projection?.error?.httpStatus === 404;
  const unreadable = projection != null && projection.data === undefined && !notFound;
  const verification = evidence?.verification;
  const acceptance = evidence?.acceptance;

  return (
    <section className="cp-region" id="section-evidence" aria-labelledby="evidence-title">
      <h3 className="cp-section-title" id="evidence-title">
        Evidence
      </h3>

      <p className="cp-region-line">
        <StateChip
          reading={{
            category: completion.disposition === "completed" ? "completed" : "unknown",
            label: completion.label,
            unmapped: false,
          }}
        />
      </p>
      <p className="cp-quiet">{completion.detail}</p>

      {notFound ? (
        <p className="cp-reason" role="status">
          <strong>No terminal evidence recorded.</strong> The daemon answered 404 for this task
          ref: there is no terminal evidence to read. That is not a failure of the task and not a
          claim that it never ran.
        </p>
      ) : null}
      {unreadable ? (
        <p className="cp-reason" role="status">
          <StateChip reading={readDomainState("load", projection.status)} /> Terminal evidence
          could not be read —{" "}
          <code className="cp-mono">{projection.error?.code ?? projection.status}</code>. This task
          is shown as neither verified nor failed.
        </p>
      ) : null}

      {verification ? (
        <>
          <h4 className="cp-section-title">Verification report</h4>
          <FactGrid
            facts={[
              {
                label: "Status",
                value: <span className="cp-mono">{verification.status ?? "unknown"}</span>,
              },
              {
                label: "Current for the fencing epoch",
                value:
                  verification.current === undefined
                    ? "unknown — the daemon did not say"
                    : verification.current
                      ? "yes"
                      : "no — this report does not prove the current state and is never read as an acceptance",
              },
              {
                label: "Completed at",
                value: (
                  <span className="cp-mono">
                    {verification.completedAt ?? "no completion timestamp recorded"}
                  </span>
                ),
              },
              {
                label: "Report ref",
                value: verification.reportRef ? (
                  <DigestChip value={verification.reportRef} label="report ref" />
                ) : (
                  "unknown"
                ),
              },
              {
                label: "Report digest",
                value: verification.reportDigest ? (
                  <DigestChip value={verification.reportDigest} label="report digest" />
                ) : (
                  "unknown"
                ),
              },
              {
                label: "Artifacts current",
                value:
                  verification.artifactsCurrent === undefined
                    ? "unknown — the daemon did not say"
                    : verification.artifactsCurrent
                      ? `yes (${verification.artifactRefs.length} artifact refs)`
                      : "no — the retained artifacts are not current for this report",
              },
            ]}
          />
        </>
      ) : (
        <p className="cp-quiet">
          The daemon holds no verification report for this task, so nothing has been independently
          verified.
        </p>
      )}

      <h4 className="cp-section-title">Terminal acceptance</h4>
      {acceptance ? (
        <FactGrid
          facts={[
            {
              label: "Current",
              value:
                acceptance.current === undefined
                  ? "unknown — the daemon did not say"
                  : acceptance.current
                    ? "yes"
                    : "no — this acceptance is not current for the fencing epoch",
            },
            {
              label: "Terminal transition ref",
              value: acceptance.terminalTransitionRef ? (
                <DigestChip
                  value={acceptance.terminalTransitionRef}
                  label="terminal transition ref"
                />
              ) : (
                "unknown"
              ),
            },
            {
              label: "Terminal transition digest",
              value: acceptance.terminalTransitionDigest ? (
                <DigestChip
                  value={acceptance.terminalTransitionDigest}
                  label="terminal transition digest"
                />
              ) : (
                "unknown"
              ),
            },
          ]}
        />
      ) : (
        <p className="cp-quiet">
          No terminal acceptance record exists. Without one this task is not complete, however a
          verification report reads.
        </p>
      )}

      <HonestyNote>
        Completion is an acceptance decision, not a test result. This section says{" "}
        <code>completed</code> only where a verification report passed, is current for the task
        fencing epoch, <em>and</em> a current terminal acceptance record exists. Every ref and
        digest above is copyable so it can be checked against the daemon.
      </HonestyNote>
    </section>
  );
}

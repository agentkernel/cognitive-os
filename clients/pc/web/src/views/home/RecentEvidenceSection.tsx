import { Link } from "react-router-dom";
import { DigestChip } from "../../components/DigestChip";
import { EmptyState } from "../../components/states";
import {
  RECENT_EVIDENCE_ROW_CAP,
  evidenceDisposition,
  formatAge,
  recentEvidenceRows,
  type EvidenceRow,
} from "../../data/projections/home";
import { HonestyNote } from "../../state/HonestyNote";
import { StateChip } from "../../state/StateChip";

/**
 * R5 Recent evidence — docs/design/13 §3. The product's trust signature: it
 * shows proof, not activity. A task appears only when the daemon actually
 * holds a verification report for it, and the row states the disposition
 * (passed / failed / not current) — never a bare "completed".
 */
export function RecentEvidenceSection({
  entries,
  probedRefs,
  failures,
  taskChannelAvailable,
  nowMs,
}: {
  entries: EvidenceRow[];
  probedRefs: string[];
  failures: { taskRef: string; code: string }[];
  taskChannelAvailable: boolean;
  nowMs: number;
}) {
  const rows = recentEvidenceRows(entries).slice(0, RECENT_EVIDENCE_ROW_CAP);

  return (
    <section className="cp-region" aria-labelledby="home-evidence-title">
      <h3 className="cp-section-title" id="home-evidence-title">
        Recent evidence
      </h3>
      {!taskChannelAvailable ? (
        <p className="cp-reason" role="status">
          Not run: evidence lives on the Task channel and this page holds a management session
          only. Issue a Task session to read it — nothing is inferred in the meantime.
        </p>
      ) : null}
      {rows.length > 0 ? (
        <ul className="cp-queue" aria-labelledby="home-evidence-title">
          {rows.map((row) => {
            const disposition = evidenceDisposition(row.view);
            const completedMs = row.view.completedAt
              ? Date.parse(row.view.completedAt)
              : Number.NaN;
            return (
              <li className="cp-queue-row" key={row.taskRef}>
                <span className="cp-queue-state">
                  <StateChip reading={disposition.reading} />
                </span>
                <span className="cp-queue-object">
                  <span className="cp-quiet">task</span>{" "}
                  <code className="cp-mono" title={row.taskRef}>
                    {row.shortRef}
                  </code>
                  {row.view.reportDigest ? (
                    <>
                      {" "}
                      <DigestChip value={row.view.reportDigest} label="report digest" />
                    </>
                  ) : null}
                </span>
                <span className="cp-queue-reason">
                  {disposition.detail}
                  {row.view.lifecycleState
                    ? ` Lifecycle state: ${row.view.lifecycleState}.`
                    : ""}
                  {row.view.artifactsCurrent === false
                    ? " Artifact evidence is no longer current."
                    : ""}
                </span>
                <span className="cp-quiet cp-queue-age">
                  {Number.isFinite(completedMs)
                    ? formatAge(completedMs, nowMs)
                    : "completion time unknown"}
                </span>
                <span className="cp-queue-action">
                  <Link to="/work">View evidence</Link>
                </span>
              </li>
            );
          })}
        </ul>
      ) : null}
      {taskChannelAvailable && rows.length === 0 ? (
        <EmptyState title="No verified outcome to show">
          None of the {probedRefs.length} task ref{probedRefs.length === 1 ? "" : "s"} this page
          knows about carries a verification report yet. Work that finished without evidence is
          deliberately absent here — it appears in Current work or the queue with its honest
          state.
        </EmptyState>
      ) : null}
      {failures.length > 0 ? (
        <p className="cp-reason" role="status">
          Evidence could not be read for {failures.length} known task ref
          {failures.length === 1 ? "" : "s"}:{" "}
          {failures.map((failure) => `${failure.taskRef} (${failure.code})`).join(", ")}. Those
          tasks are not shown as verified and are not shown as failed.
        </p>
      ) : null}
      <HonestyNote>
        Coverage is per task ref: the daemon has no evidence stream, so this region probes only
        the task refs the page already knows about (envelope list plus refs observed this
        session). A completion never renders without its verification report, and a report that
        is not current for the task&apos;s fencing epoch is shown as unproven rather than as
        success.
      </HonestyNote>
    </section>
  );
}

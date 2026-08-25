import { Link } from "react-router-dom";
import { DigestChip } from "../../components/DigestChip";
import { EmptyState } from "../../components/states";
import {
  CURRENT_WORK_ROW_CAP,
  currentWorkReading,
  formatAge,
  mergeCurrentWork,
  taskListAtBound,
  TASK_LIST_LIMIT,
  type ObservedTask,
  type TaskEnvelopeView,
} from "../../data/projections/home";
import type { Projection } from "../../data/store";
import type { LastGood } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { StateChip } from "../../state/StateChip";
import { RegionStatus } from "./RegionStatus";

const ORIGIN_LABEL: Record<string, string> = {
  envelope: "daemon envelope list",
  session: "observed this session",
  "envelope+session": "daemon envelope list · observed this session",
};

/**
 * R3 Current work — docs/design/13 §3. The daemon's task list is an envelope
 * projection: current-epoch contract rows with no lifecycle state and no
 * objective (BD-3). This region merges it with the refs this browser session
 * actually observed and says exactly that, permanently. It is a
 * capability-honesty feature, not an apology.
 */
export function CurrentWorkSection({
  projection,
  lastGood,
  observed,
  nowMs,
}: {
  projection: Projection<TaskEnvelopeView[]>;
  lastGood: LastGood<TaskEnvelopeView[]>;
  observed: ObservedTask[];
  nowMs: number;
}) {
  const envelopes = lastGood.data ?? [];
  const merged = mergeCurrentWork(envelopes, observed);
  const rows = merged.slice(0, CURRENT_WORK_ROW_CAP);
  const hidden = merged.length - rows.length;
  const authoritativelyEmpty = lastGood.live && merged.length === 0;

  return (
    <section className="cp-region" aria-labelledby="home-work-title">
      <h3 className="cp-section-title" id="home-work-title">
        Current work
      </h3>
      <RegionStatus projection={projection} lastGood={lastGood} what="the task list" />
      {rows.length > 0 ? (
        <ul className="cp-queue" aria-labelledby="home-work-title">
          {rows.map((row) => (
            <li className="cp-queue-row" key={row.taskRef} data-origin={row.origin}>
              <span className="cp-queue-state">
                <StateChip reading={currentWorkReading(row)} />
              </span>
              <span className="cp-queue-object">
                <span className="cp-quiet">task</span>{" "}
                <code className="cp-mono" title={row.taskRef}>
                  {row.shortRef}
                </code>{" "}
                <DigestChip value={row.taskRef} label="task ref" />
              </span>
              <span className="cp-queue-reason">
                {row.objective ?? "objective is not exposed by the daemon's task list"}
                {row.contractEpoch != null ? ` · contract epoch ${row.contractEpoch}` : ""}
                {` · source: ${ORIGIN_LABEL[row.origin] ?? row.origin}`}
              </span>
              <span className="cp-quiet cp-queue-age">
                {row.observedAtMs != null
                  ? `observed ${formatAge(row.observedAtMs, nowMs)}`
                  : "age unknown (the envelope carries no timestamp)"}
              </span>
              <span className="cp-queue-action">
                <Link to="/work">Open Work</Link>
              </span>
            </li>
          ))}
        </ul>
      ) : null}
      {authoritativelyEmpty ? (
        <EmptyState
          title="No work observed yet"
          action={<Link to="/work">Create a task in Work</Link>}
        >
          The daemon lists no current task contracts and this session has observed no task refs.
          This is an authoritative empty, not a loading placeholder.
        </EmptyState>
      ) : null}
      {hidden > 0 ? (
        <p className="cp-next">
          <Link to="/work">{hidden} more in Work</Link>
        </p>
      ) : null}
      <HonestyNote>
        Inventory is partial — daemon task listing is envelope-only. The list carries a task
        ref, a contract epoch and a digest; it carries no lifecycle state and no objective
        (BD-3), so no row here claims to be running. Rows also include task refs this browser
        session observed, which is memory of this tab, not an inventory.
        {taskListAtBound(envelopes)
          ? ` The daemon list is at its ${TASK_LIST_LIMIT}-row bound, so further contracts may exist without appearing here.`
          : ` The daemon list is bounded to ${TASK_LIST_LIMIT} rows.`}{" "}
        The task watch snapshot does not represent current task inventory: its snapshot list is
        always empty and its event ring is process-local, so it is not used as a source here.
        Per-task detail lands with the Work space (waves 4–5); today a row opens the Work space
        and its full ref is copyable for the governed-task flow.
      </HonestyNote>
    </section>
  );
}

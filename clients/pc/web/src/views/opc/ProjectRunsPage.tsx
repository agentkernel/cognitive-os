import { useCallback, useEffect } from "react";
import { Link, useParams } from "react-router-dom";
import { PageHeader } from "../../components/PageHeader";
import {
  PROJECT_AXIS_PATH,
  projectAxisKey,
  type ProjectAxisStageRow,
} from "../../data/projections/projectWork";
import {
  ATTEMPT_LIST_PATH,
  attemptHistoryKey,
  ROUTINE_RUNS_PATH,
  routineRunsKey,
  type AttemptHistoryRow,
  type RoutineRunsView,
} from "../../data/projections/routineRuns";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { loadAttemptHistory, loadProjectAxis, loadRoutineRuns } from "./loadOpcReads";
import { ProjectWorkNav } from "./ProjectWorkNav";

/**
 * Runs — PlanRevision axis (P12-T03) + the real Routine occurrence ledger and
 * Attempt history (P13-T05/D02). Every row is a daemon fact: the scheduler
 * tick is the only dispatcher, a receipt is not completion, verification is
 * `not-run`. There is no Start, Approve, or Complete control on this page;
 * manual triggers stay on the management `routine.trigger` Intent route.
 */
export function ProjectRunsPage() {
  const { projectId = "" } = useParams();
  const axis = useProjection<ProjectAxisStageRow[]>(projectAxisKey(projectId));
  const runs = useProjection<RoutineRunsView[]>(routineRunsKey(projectId));
  const attempts = useProjection<AttemptHistoryRow[]>(attemptHistoryKey(projectId));
  const refresh = useCallback(async () => {
    if (projectId.length === 0) {
      return;
    }
    await Promise.all([
      loadProjectAxis(projectId),
      loadRoutineRuns(projectId),
      loadAttemptHistory(projectId),
    ]);
  }, [projectId]);
  useEffect(() => {
    void refresh();
  }, [refresh]);

  const view = runs.status === "ready" || runs.status === "stale" ? runs.data?.[0] : undefined;

  return (
    <section data-page="opc-project-runs">
      <PageHeader
        title="Project runs"
        lede="Routine occurrence ledger, Attempt history, and the current PlanRevision axis. Not a renamed Work timeline."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. GET {ROUTINE_RUNS_PATH} is the
        occurrence ledger the daemon scheduler tick writes; GET {ATTEMPT_LIST_PATH}{" "}
        is the Attempt history. A receipt or process exit is never completion and
        verification stays not-run. No Start, Approve, or Complete control lives
        here; manual triggers are Intent on the management channel.
      </HonestyNote>
      <p className="cp-quiet">
        <Link to="/projects">Projects list</Link>
        {projectId ? (
          <>
            {" "}
            · <code className="cp-mono">{projectId}</code>
          </>
        ) : null}
      </p>
      {projectId ? <ProjectWorkNav projectId={projectId} /> : null}

      <DaemonReadPanel
        projection={runs}
        surface="Routine occurrence ledger"
        emptyTitle="Project runs: no occurrence ledger"
        emptyBody="The daemon returned no ledger for this Project. Nothing is inferred as an empty run list and no routine_id is invented."
        region="opc-routine-runs"
      >
        {view ? <RoutineLedger view={view} /> : null}
      </DaemonReadPanel>

      <DaemonReadPanel
        projection={attempts}
        surface="Attempt history"
        emptyTitle="Attempt history: no hosted Attempt yet"
        emptyBody="No hosted Attempt exists for this Project. An occurrence becomes an Attempt only when the daemon tick leases and launches it; this empty history is a fact, not a failure."
        region="opc-attempt-history"
      >
        <table className="cp-table" data-region="opc-attempt-history-table">
          <caption className="cp-quiet">
            GET {ATTEMPT_LIST_PATH} — daemon-observed terminals; receipt is not completion
          </caption>
          <thead>
            <tr>
              <th>Attempt</th>
              <th>Member</th>
              <th>Task ref</th>
              <th>State</th>
              <th>Terminal</th>
              <th>Response</th>
              <th>Completion claimed</th>
              <th>Verification</th>
              <th>Elapsed</th>
            </tr>
          </thead>
          <tbody>
            {(attempts.data ?? []).map((row) => (
              <tr key={row.attemptId} data-row-key={row.attemptId} data-attempt={row.attemptId}>
                <td>
                  <code className="cp-mono">{row.attemptId}</code>
                </td>
                <td>
                  <code className="cp-mono">{row.employeeId}</code>
                </td>
                <td>
                  <code className="cp-mono">{row.taskRef}</code>
                </td>
                <td>{row.state}</td>
                <td>
                  {row.terminalKind}
                  {row.exitCode !== "—" ? ` / exit ${row.exitCode}` : ""}
                </td>
                <td>{row.responseStatus}</td>
                <td>{row.completionClaimed}</td>
                <td>{row.verificationStatus}</td>
                <td>{row.elapsedMs === "—" ? "—" : `${row.elapsedMs} ms`}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </DaemonReadPanel>

      <DaemonReadPanel
        projection={axis}
        surface="PlanRevision axis"
        emptyTitle="Project runs: no PlanRevision axis"
        emptyBody="No daemon stages. This is not an empty Work list and not a fake run."
        region="opc-project-runs"
      >
        <table className="cp-table">
          <caption className="cp-quiet">GET {PROJECT_AXIS_PATH}</caption>
          <thead>
            <tr>
              <th>Position</th>
              <th>Stage</th>
              <th>Confirm</th>
              <th>Ready</th>
              <th>Seated</th>
              <th>Gaps</th>
            </tr>
          </thead>
          <tbody>
            {(axis.data ?? []).map((stage) => (
              <tr key={stage.stageId} data-row-key={stage.stageId}>
                <td>{stage.position}</td>
                <td>
                  <code className="cp-mono">{stage.stageId}</code> {stage.title}
                </td>
                <td>{stage.confirmStatus}</td>
                <td>{stage.ready}</td>
                <td>{stage.seated}</td>
                <td>{stage.gapCount}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </DaemonReadPanel>
    </section>
  );
}

function RoutineLedger({ view }: { view: RoutineRunsView }) {
  const summary = view.summary;
  return (
    <div data-region="opc-routine-ledger">
      <p className="cp-quiet" data-region="opc-routine-runs-summary">
        Scheduler <code className="cp-mono">{view.scheduler}</code> · host available{" "}
        <code className="cp-mono">{view.hostAvailable}</code>
        {view.hostReason !== "—" ? (
          <>
            {" "}
            (<code className="cp-mono">{view.hostReason}</code>)
          </>
        ) : null}{" "}
        · active {summary.active} · running {summary.running} · queued {summary.queued} ·
        missed {summary.missed} · coalesced {summary.coalesced} · attempted{" "}
        {summary.attempted} (done {summary.done} · failed {summary.failed} · unknown{" "}
        {summary.unknown}). Counts are daemon-stated; unknown is never 0. Verification{" "}
        {view.verificationStatus}; clock / sleep / restart host E2E{" "}
        {view.clockSleepRestartHostE2e}.
      </p>

      <table className="cp-table" data-region="opc-routine-armings">
        <caption className="cp-quiet">
          Armed Routines — armed after G2 from the ③ 周期与触发 declaration; the daemon tick
          is the only dispatcher
        </caption>
        <thead>
          <tr>
            <th>Arming</th>
            <th>Routine / revision</th>
            <th>Stage</th>
            <th>Member</th>
            <th>Cadence</th>
            <th>State</th>
            <th>Apply</th>
            <th>Next due</th>
          </tr>
        </thead>
        <tbody>
          {view.armings.length === 0 ? (
            <tr data-row-key="no-arming">
              <td colSpan={8} className="cp-quiet">
                No armed Routine. Nothing is scheduled for this Project; an occurrence
                triggered now would land as missed / not-armed, never vanish.
              </td>
            </tr>
          ) : null}
          {view.armings.map((row) => (
            <tr key={row.armingId} data-row-key={row.armingId} data-arming={row.armingId}>
              <td>
                <code className="cp-mono">{row.armingId}</code>
              </td>
              <td>
                <code className="cp-mono">{row.routineId}</code> /{" "}
                <code className="cp-mono">{row.revisionId}</code>
              </td>
              <td>
                <code className="cp-mono">{row.stageId}</code>
              </td>
              <td>
                <code className="cp-mono">{row.employeeId}</code>
              </td>
              <td>
                {row.cadenceKind}
                {row.intervalMs !== "—" ? ` · ${row.intervalMs} ms` : ""}
              </td>
              <td>{row.state}</td>
              <td>{row.applyMode}</td>
              <td>{row.nextDueAt}</td>
            </tr>
          ))}
        </tbody>
      </table>

      <table className="cp-table" data-region="opc-routine-occurrences">
        <caption className="cp-quiet">
          GET {ROUTINE_RUNS_PATH} — occurrence ledger (no-overlap · queue-latest · missed ·
          coalesced). Working is not completion.
        </caption>
        <thead>
          <tr>
            <th>Occurrence</th>
            <th>Routine / revision</th>
            <th>Trigger</th>
            <th>Disposition</th>
            <th>Dispatch</th>
            <th>Ledger reason</th>
            <th>Attempt</th>
            <th>Outcome</th>
            <th>Elapsed</th>
          </tr>
        </thead>
        <tbody>
          {view.occurrences.length === 0 ? (
            <tr data-row-key="no-occurrence">
              <td colSpan={9} className="cp-quiet">
                No occurrence recorded. This is the daemon ledger, not an empty run list.
              </td>
            </tr>
          ) : null}
          {view.occurrences.map((row) => (
            <tr
              key={row.occurrenceId}
              data-row-key={row.occurrenceId}
              data-occurrence={row.occurrenceId}
              data-disposition={row.disposition}
              data-dispatch-state={row.dispatchState}
            >
              <td>
                <code className="cp-mono">{row.occurrenceId}</code>
              </td>
              <td>
                <code className="cp-mono">{row.routineId}</code> /{" "}
                <code className="cp-mono">{row.revisionId}</code>
              </td>
              <td>
                {row.triggerKind} / {row.triggerSource}
              </td>
              <td>{row.disposition}</td>
              <td>{row.dispatchState}</td>
              <td>
                {row.coalescedBy !== "—" ? (
                  <>
                    coalesced by <code className="cp-mono">{row.coalescedBy}</code>
                  </>
                ) : row.missReason !== "—" ? (
                  <>
                    missed: <code className="cp-mono">{row.missReason}</code>
                  </>
                ) : (
                  "—"
                )}
              </td>
              <td>
                {row.attemptId !== "—" ? <code className="cp-mono">{row.attemptId}</code> : "—"}
              </td>
              <td>
                {row.attemptOutcome}
                {row.completionClaimed === "true" ? " · completion_claimed=true (contract violation)" : ""}
              </td>
              <td>{row.elapsedMs === "—" ? "—" : `${row.elapsedMs} ms`}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

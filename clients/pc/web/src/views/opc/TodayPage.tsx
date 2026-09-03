import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { PageHeader } from "../../components/PageHeader";
import { hitlCanvasPath, HITL_KEY, type PendingPreviewRow } from "../../data/projections/hitl";
import {
  creatingProjectRows,
  liveProjectRows,
  PROJECTS_KEY,
  PROJECT_LIST_PATH,
  type ProjectListRow,
} from "../../data/projections/projects";
import {
  formatDuration,
  TODAY_OVERVIEW_KEY,
  TODAY_OVERVIEW_PATH,
  TODAY_PERIODS,
  type TodayOverviewView,
  type TodayPeriod,
} from "../../data/projections/todayOverview";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import {
  loadPendingPreviewsForLiveProject,
  loadProjectList,
  loadTodayOverviewForLiveProject,
  liveProjectId,
} from "./loadOpcReads";
import {
  ProjectAuthorityPanel,
  TODAY_EMPTY_ONLY_CREATE,
  TODAY_INCOMPLETE_ONLY_CREATE,
} from "./ProjectAuthorityPanel";

const PERIOD_LABEL: Record<TodayPeriod, string> = {
  today: "Today",
  week: "This week",
  month: "This month",
};

/**
 * Today — Personal 2.0. Empty home is only-create (P12-T02). Creating-only
 * is continue-create (today-incomplete). Live Projects get daily packets
 * from pending-previews, deep-linked to the HITL canvas, and the run
 * overview from `today.overview` (P13-T05/D02): one row per live Project
 * (state · completed Attempts · current stage · duration) plus created /
 * live / blocked counts and a today / week / month switch. With nothing
 * pending the packet block collapses and the overview stays. No KPI wall;
 * unknown is never 0; chat has no Approve. T06 Confirm stays on management HTTP.
 */
export function TodayPage() {
  const projects = useProjection<ProjectListRow[]>(PROJECTS_KEY);
  const hitl = useProjection<PendingPreviewRow[]>(HITL_KEY);
  const overview = useProjection<TodayOverviewView[]>(TODAY_OVERVIEW_KEY);
  const [period, setPeriod] = useState<TodayPeriod>("today");
  const liveId = liveProjectId(projects);
  const liveRows = liveProjectRows(projects.data);
  const creatingRows = creatingProjectRows(projects.data);
  const refresh = useCallback(async () => {
    const list = await loadProjectList();
    await Promise.all([
      loadPendingPreviewsForLiveProject(list),
      loadTodayOverviewForLiveProject(list, period),
    ]);
  }, [period]);
  useEffect(() => {
    void refresh();
  }, [refresh]);

  const incompleteOnly =
    projects.status === "ready" && liveRows.length === 0 && creatingRows.length > 0;
  const packets = hitl.status === "ready" ? (hitl.data ?? []) : [];
  const packetsCollapsed = hitl.status === "empty" || (hitl.status === "ready" && packets.length === 0);
  const emptyHome =
    projects.status === "empty" ||
    (projects.status === "ready" && (projects.data?.length ?? 0) === 0);
  const lede = emptyHome
    ? "Start create. Not Home, not an Inbox, not a decision packet."
    : incompleteOnly
      ? "Create is not finished. Daily packets wait for activation."
      : "What needs the Owner on a live Project, and how its Routines ran. Not Home, not an Inbox, not a KPI wall.";
  const view =
    overview.status === "ready" || overview.status === "stale" ? overview.data?.[0] : undefined;

  return (
    <section data-page="opc-today">
      <PageHeader title="Today" lede={lede} />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. Vite is not the product origin.
        Empty home is only-create. Creating Projects stay continue-create. Daily
        packets and the run overview exist only for live daemon state. Completed
        runs are daemon-observed Attempt terminals, not verified completion. Chat
        cannot Approve.
      </HonestyNote>
      <ProjectAuthorityPanel
        projection={projects}
        surface="Today"
        emptyBody={TODAY_EMPTY_ONLY_CREATE}
        emptyAction={
          <Link className="cp-button cp-button--primary" to="/projects/new">
            Start create
          </Link>
        }
      >
        {incompleteOnly ? (
          <div data-surface="today-incomplete">
            <p>{TODAY_INCOMPLETE_ONLY_CREATE}</p>
            <p>
              <Link className="cp-button cp-button--primary" to="/projects/new">
                Continue create
              </Link>
            </p>
            <table className="cp-table">
              <caption className="cp-quiet">
                GET {PROJECT_LIST_PATH} — creating only; not a packet
              </caption>
              <thead>
                <tr>
                  <th>Project</th>
                  <th>State</th>
                  <th>Title</th>
                  <th>Cost</th>
                </tr>
              </thead>
              <tbody>
                {creatingRows.map((row) => (
                  <tr key={row.projectId} data-row-key={row.projectId}>
                    <td>
                      <code className="cp-mono">{row.projectId}</code>
                    </td>
                    <td>{row.state}</td>
                    <td>{row.titleSummary}</td>
                    <td>{row.cost}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div data-surface="today">
            <p className="cp-quiet">
              {liveRows.length} live Project
              {liveRows.length === 1 ? "" : "s"} on{" "}
              <code className="cp-mono">{PROJECT_LIST_PATH}</code>. This overview
              is daemon-stated rows, not a KPI wall. Unknown cost stays unknown.
            </p>
            {creatingRows.length > 0 ? (
              <p>
                Unaccepted creating Projects stay continue-create, not a daily
                packet.{" "}
                <Link to="/projects/new">Continue create</Link>
              </p>
            ) : null}
            {liveId ? (
              <details
                data-region="opc-today-packet-block"
                data-collapsed={packetsCollapsed ? "true" : "false"}
                open={!packetsCollapsed}
              >
                <summary className="cp-quiet">
                  {packetsCollapsed
                    ? "Decision packet: nothing pending — collapsed; the run overview stays."
                    : `Decision packet: ${packets.length} pending ApprovalPreview${packets.length === 1 ? "" : "s"}.`}
                </summary>
                <DaemonReadPanel
                  projection={hitl}
                  surface="Today decision packets"
                  emptyTitle="Today: no pending ApprovalPreview"
                  emptyBody="No pending ApprovalPreview for a live Project. Chat cannot Approve. Confirm stays on management HTTP. This empty packet slot is not a KPI."
                  region="opc-hitl"
                >
                  {packets.length === 0 ? null : (
                    <div data-region="opc-today-packets">
                      {packets.map((row) => (
                        <article
                          key={row.previewId}
                          className="cp-region"
                          data-packet={row.previewId}
                          data-row-key={row.previewId}
                        >
                          <h3>Needs owner decision</h3>
                          <p className="cp-quiet">
                            Announce only. Reversible until management confirm.
                            Narrowing needs a new preview. Chat cannot Approve.
                            Cost unknown is not 0. This is not a KPI.
                          </p>
                          <dl>
                            <div>
                              <dt>Preview</dt>
                              <dd>
                                <code className="cp-mono">{row.previewId}</code>
                              </dd>
                            </div>
                            <div>
                              <dt>Kind</dt>
                              <dd>{row.subjectKind}</dd>
                            </div>
                            <div>
                              <dt>Status</dt>
                              <dd>{row.status}</dd>
                            </div>
                          </dl>
                          <p>
                            <Link
                              className="cp-button cp-button--primary"
                              to={hitlCanvasPath(row.previewId, liveId)}
                            >
                              Open this decision on the canvas
                            </Link>
                          </p>
                        </article>
                      ))}
                    </div>
                  )}
                </DaemonReadPanel>
              </details>
            ) : null}

            {liveId ? (
              <div data-region="opc-today-overview-block">
                <p
                  className="cp-quiet"
                  role="group"
                  aria-label="Overview period"
                  data-region="opc-today-period"
                >
                  {TODAY_PERIODS.map((candidate) => (
                    <button
                      key={candidate}
                      type="button"
                      className="cp-button"
                      aria-pressed={period === candidate}
                      data-period={candidate}
                      onClick={() => setPeriod(candidate)}
                    >
                      {PERIOD_LABEL[candidate]}
                    </button>
                  ))}
                </p>
                <DaemonReadPanel
                  projection={overview}
                  surface="Today run overview"
                  emptyTitle="Today: no run overview"
                  emptyBody="The daemon returned no overview for this period. Nothing is inferred as zero runs."
                  region="opc-today-overview"
                >
                  {view ? <RunOverview view={view} /> : null}
                </DaemonReadPanel>
              </div>
            ) : null}

            <table className="cp-table" data-region="opc-today-live-list">
              <caption className="cp-quiet">
                Live Projects — daemon list, not a KPI wall
              </caption>
              <thead>
                <tr>
                  <th>Project</th>
                  <th>State</th>
                  <th>Title</th>
                  <th>Cost</th>
                </tr>
              </thead>
              <tbody>
                {liveRows.map((row) => (
                  <tr key={row.projectId} data-row-key={row.projectId}>
                    <td>
                      <code className="cp-mono">{row.projectId}</code>{" "}
                      <Link to={`/projects/${encodeURIComponent(row.projectId)}`}>
                        Open
                      </Link>
                    </td>
                    <td>{row.state}</td>
                    <td>{row.titleSummary}</td>
                    <td>{row.cost}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </ProjectAuthorityPanel>
    </section>
  );
}

function RunOverview({ view }: { view: TodayOverviewView }) {
  return (
    <div data-region="opc-today-run-overview" data-period={view.period}>
      <p className="cp-quiet" data-region="opc-today-counts">
        Period <code className="cp-mono">{view.period}</code> ({view.periodBasis}) · created{" "}
        <span data-count="created">{view.created}</span> · live{" "}
        <span data-count="live">{view.live}</span> · blocked{" "}
        <span data-count="blocked">{view.blocked}</span>. Counts are daemon-stated; unknown is
        never 0. Verification {view.verificationStatus}; cost {view.cost}.
      </p>
      <table className="cp-table" data-region="opc-today-overview-rows">
        <caption className="cp-quiet">
          GET {TODAY_OVERVIEW_PATH} — one row per live Project; completed runs are
          daemon-observed Attempt terminals, not verified completion
        </caption>
        <thead>
          <tr>
            <th>Project</th>
            <th>Status</th>
            <th>Completed runs</th>
            <th>Current stage</th>
            <th>Duration</th>
            <th>Queued / missed</th>
            <th>Failed / unknown</th>
          </tr>
        </thead>
        <tbody>
          {view.rows.length === 0 ? (
            <tr data-row-key="no-live-row">
              <td colSpan={7} className="cp-quiet">
                No live Project row in this period. This is the daemon answer, not zero.
              </td>
            </tr>
          ) : null}
          {view.rows.map((row) => (
            <tr
              key={row.projectId}
              data-row-key={`overview:${row.projectId}`}
              data-overview-project={row.projectId}
              data-status={row.status}
            >
              <td>
                <code className="cp-mono">{row.projectId}</code>{" "}
                <Link to={`/projects/${encodeURIComponent(row.projectId)}/runs`}>Runs</Link>
              </td>
              <td>
                {row.status} · {row.state}
              </td>
              <td>{row.attemptsDone}</td>
              <td>
                {row.currentStageId !== "—" ? (
                  <>
                    <code className="cp-mono">{row.currentStageId}</code>{" "}
                    {row.currentStageTitle !== "—" ? row.currentStageTitle : ""}
                  </>
                ) : (
                  "—"
                )}
              </td>
              <td>{formatDuration(row.durationMs)}</td>
              <td>
                {row.queuedCount} / {row.missedCount}
              </td>
              <td>
                {row.attemptsFailed} / {row.attemptsUnknown}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

import { useCallback, useEffect } from "react";
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
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { loadPendingPreviewsForLiveProject, loadProjectList, liveProjectId } from "./loadOpcReads";
import {
  ProjectAuthorityPanel,
  TODAY_EMPTY_ONLY_CREATE,
  TODAY_INCOMPLETE_ONLY_CREATE,
} from "./ProjectAuthorityPanel";

/**
 * Today — Personal 2.0. Empty home is only-create (P12-T02). Creating-only
 * is continue-create (today-incomplete). Live Projects get daily packets
 * from pending-previews, deep-linked to the HITL canvas. No KPI wall.
 * Chat has no Approve. T06 Confirm stays on management HTTP.
 */
export function TodayPage() {
  const projects = useProjection<ProjectListRow[]>(PROJECTS_KEY);
  const hitl = useProjection<PendingPreviewRow[]>(HITL_KEY);
  const liveId = liveProjectId(projects);
  const liveRows = liveProjectRows(projects.data);
  const creatingRows = creatingProjectRows(projects.data);
  const refresh = useCallback(async () => {
    const list = await loadProjectList();
    await loadPendingPreviewsForLiveProject(list);
  }, []);
  useEffect(() => {
    void refresh();
  }, [refresh]);

  const incompleteOnly =
    projects.status === "ready" && liveRows.length === 0 && creatingRows.length > 0;
  const packets = hitl.status === "ready" ? (hitl.data ?? []) : [];
  const emptyHome =
    projects.status === "empty" ||
    (projects.status === "ready" && (projects.data?.length ?? 0) === 0);
  const lede = emptyHome
    ? "Start create. Not Home, not an Inbox, not a decision packet."
    : incompleteOnly
      ? "Create is not finished. Daily packets wait for activation."
      : "What needs the Owner on a live Project. Not Home, not an Inbox, not a KPI wall.";

  return (
    <section data-page="opc-today">
      <PageHeader title="Today" lede={lede} />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. Vite is not the product origin.
        Empty home is only-create. Creating Projects stay continue-create. Daily
        packets exist only for live daemon state. Chat cannot Approve.
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
            ) : null}
            <table className="cp-table">
              <caption className="cp-quiet">
                Live overview — daemon list, not a KPI wall
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

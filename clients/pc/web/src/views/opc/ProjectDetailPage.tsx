import { useCallback, useEffect } from "react";
import { Link, useParams, useSearchParams } from "react-router-dom";
import { PageHeader } from "../../components/PageHeader";
import {
  HITL_KEY,
  type PendingPreviewRow,
} from "../../data/projections/hitl";
import {
  projectAxisKey,
  PROJECT_AXIS_PATH,
  PROJECT_DETAIL_PATH,
  projectDetailKey,
  type ProjectAxisStageRow,
  type ProjectDetailRow,
} from "../../data/projections/projectWork";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { HitlCanvasTable } from "./HitlCanvasTable";
import {
  loadPendingPreviewsForProject,
  loadProjectAxis,
  loadProjectDetail,
} from "./loadOpcReads";
import { ProjectWorkNav } from "./ProjectWorkNav";

/**
 * Project detail — daemon GET detail + read-only PlanRevision axis.
 * L2 goes to members/runs/outputs. HITL here is announce-only (T06 Confirm).
 */
export function ProjectDetailPage() {
  const { projectId = "" } = useParams();
  const [params] = useSearchParams();
  const focusPreviewId = params.get("preview");
  const detail = useProjection<ProjectDetailRow[]>(projectDetailKey(projectId));
  const axis = useProjection<ProjectAxisStageRow[]>(projectAxisKey(projectId));
  const hitl = useProjection<PendingPreviewRow[]>(`${HITL_KEY}:${projectId}`);
  const refresh = useCallback(async () => {
    if (projectId.length === 0) {
      return;
    }
    await loadProjectDetail(projectId);
    await loadProjectAxis(projectId);
    await loadPendingPreviewsForProject(projectId);
  }, [projectId]);
  useEffect(() => {
    void refresh();
  }, [refresh]);
  const row = detail.data?.[0];

  return (
    <section data-page="opc-project-detail">
      <PageHeader
        title="Project detail"
        lede="Daemon Project aggregate. Not a renamed Task."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. Vite is not the product origin.
        GET {PROJECT_DETAIL_PATH} is the header. The process axis is read-only.
        Confirm-before-activate stays on management HTTP. Chat cannot Approve.
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
        projection={detail}
        surface="Project detail"
        emptyTitle="Project detail: no Project"
        emptyBody="This hash is not a daemon Project. It is not a Task ref renamed as a Project."
        region="opc-project-detail"
      >
        {row ? (
          <table className="cp-table">
            <caption className="cp-quiet">GET {PROJECT_DETAIL_PATH}</caption>
            <thead>
              <tr>
                <th>Field</th>
                <th>Daemon statement</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td>State</td>
                <td>{row.state}</td>
              </tr>
              <tr>
                <td>Charter</td>
                <td>{row.charterStatus}</td>
              </tr>
              <tr>
                <td>Charter digest</td>
                <td>
                  <code className="cp-mono">{row.charterDigest}</code>
                </td>
              </tr>
              <tr>
                <td>Plan revision</td>
                <td>
                  <code className="cp-mono">{row.planRevisionId}</code>
                </td>
              </tr>
              <tr>
                <td>Cost</td>
                <td>{row.cost}</td>
              </tr>
              <tr>
                <td>Pending previews</td>
                <td>{row.pendingPreviewCount}</td>
              </tr>
            </tbody>
          </table>
        ) : null}
      </DaemonReadPanel>
      <DaemonReadPanel
        projection={axis}
        surface="Project process axis"
        emptyTitle="Project detail: no PlanRevision axis"
        emptyBody="Missing plan is empty, not a fake wizard. Runs and outputs stay empty until the daemon states stages."
        region="opc-project-axis"
      >
        <table className="cp-table">
          <caption className="cp-quiet">GET {PROJECT_AXIS_PATH} — read-only</caption>
          <thead>
            <tr>
              <th>Stage</th>
              <th>Title</th>
              <th>Confirm</th>
              <th>Ready</th>
            </tr>
          </thead>
          <tbody>
            {(axis.data ?? []).map((stage) => (
              <tr key={stage.stageId} data-row-key={stage.stageId}>
                <td>
                  <code className="cp-mono">{stage.stageId}</code>
                </td>
                <td>{stage.title}</td>
                <td>{stage.confirmStatus}</td>
                <td>{stage.ready}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </DaemonReadPanel>
      {projectId ? (
        <DaemonReadPanel
          projection={hitl}
          surface="Project HITL canvas"
          emptyTitle="Project: no pending ApprovalPreview"
          emptyBody="No pending ApprovalPreview. Chat cannot Approve. This canvas does not mint Confirm."
          region="opc-hitl"
        >
          <HitlCanvasTable
            projectId={projectId}
            rows={hitl.data ?? []}
            focusPreviewId={focusPreviewId}
            deepLink={false}
          />
        </DaemonReadPanel>
      ) : null}
    </section>
  );
}

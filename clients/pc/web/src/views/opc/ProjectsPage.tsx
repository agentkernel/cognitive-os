import { useCallback, useEffect } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { PageHeader } from "../../components/PageHeader";
import { HITL_KEY, type PendingPreviewRow } from "../../data/projections/hitl";
import { PROJECTS_KEY, PROJECT_LIST_PATH, type ProjectListRow } from "../../data/projections/projects";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { HitlCanvasTable } from "./HitlCanvasTable";
import { loadPendingPreviewsForReadyProject, loadProjectList, readyProjectId } from "./loadOpcReads";
import { ProjectAuthorityPanel } from "./ProjectAuthorityPanel";

/**
 * Projects — L1 inventory of daemon Project rows plus the HITL canvas.
 * Populated only from GET /management/project/v1/list. Open is hash
 * navigation to `#/projects/:id` four submenus. No create/activate/Confirm
 * on this list. Today deep-links here via ?preview=; T06 Confirm is on this
 * canvas via preview-detail digest. Not Inbox L1.
 */
export function ProjectsPage() {
  const [params] = useSearchParams();
  const focusPreviewId = params.get("preview");
  const projects = useProjection<ProjectListRow[]>(PROJECTS_KEY);
  const hitl = useProjection<PendingPreviewRow[]>(HITL_KEY);
  const projectId = readyProjectId(projects);
  const refresh = useCallback(async () => {
    const list = await loadProjectList();
    await loadPendingPreviewsForReadyProject(list);
  }, []);
  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <section data-page="opc-projects">
      <PageHeader
        title="Projects"
        lede="Daemon Project aggregate. Not a renamed Task list."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. Vite is not the product origin.
        Rows are the daemon list. Confirm-before-activate stays on management HTTP.
        HITL on this page is the project-center canvas, not an Inbox.
      </HonestyNote>
      <ProjectAuthorityPanel projection={projects} surface="Projects">
        <table className="cp-table">
          <caption className="cp-quiet">GET {PROJECT_LIST_PATH}</caption>
          <thead>
            <tr>
              <th>Project</th>
              <th>State</th>
              <th>Title</th>
              <th>Cost</th>
            </tr>
          </thead>
          <tbody>
            {(projects.data ?? []).map((row) => (
              <tr key={row.projectId} data-row-key={row.projectId}>
                <td>
                  <code className="cp-mono">{row.projectId}</code>{" "}
                  <Link to={`/projects/${encodeURIComponent(row.projectId)}`}>Open</Link>
                </td>
                <td>{row.state}</td>
                <td>{row.titleSummary}</td>
                <td>{row.cost}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {projectId ? (
          <DaemonReadPanel
            projection={hitl}
            surface="Projects HITL canvas"
            emptyTitle="Projects: no pending ApprovalPreview"
            emptyBody="No pending ApprovalPreview. Chat cannot Approve. Confirm stays on this canvas when preview-detail supplies a digest."
            region="opc-hitl"
          >
            <HitlCanvasTable
              projectId={projectId}
              rows={hitl.data ?? []}
              focusPreviewId={focusPreviewId}
              deepLink={false}
              onWritten={() => {
                void refresh();
              }}
            />
          </DaemonReadPanel>
        ) : null}
      </ProjectAuthorityPanel>
    </section>
  );
}

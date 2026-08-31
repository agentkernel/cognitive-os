import { useCallback, useEffect } from "react";
import { PageHeader } from "../../components/PageHeader";
import { fetchProjection } from "../../data/fetchProjection";
import { HITL_KEY, pendingPreviewsPath, projectPendingPreviews, type PendingPreviewRow } from "../../data/projections/hitl";
import { PROJECTS_KEY, PROJECT_LIST_PATH, type ProjectListRow } from "../../data/projections/projects";
import { appProjections } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { loadProjectList, readyProjectId } from "./loadOpcReads";
import { ProjectAuthorityPanel } from "./ProjectAuthorityPanel";

/**
 * Projects — L1 inventory of daemon Project rows plus HITL announce-only.
 * No create/activate/Confirm control in this slice.
 */
export function ProjectsPage() {
  const projects = useProjection<ProjectListRow[]>(PROJECTS_KEY);
  const hitl = useProjection<PendingPreviewRow[]>(HITL_KEY);
  const projectId = readyProjectId(projects);
  const refresh = useCallback(async () => {
    const list = await loadProjectList();
    const id = readyProjectId(list);
    if (!id) {
      return;
    }
    await fetchProjection(
      appProjections,
      HITL_KEY,
      pendingPreviewsPath(id),
      "management",
      projectPendingPreviews,
    );
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
                  <code className="cp-mono">{row.projectId}</code>
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
            surface="Projects HITL announcements"
            emptyTitle="Projects: no pending ApprovalPreview"
            emptyBody="No pending ApprovalPreview. Chat cannot Approve."
            region="opc-hitl"
          >
            <p className="cp-quiet">
              {(hitl.data?.length ?? 0)} pending ApprovalPreview
              {(hitl.data?.length ?? 0) === 1 ? "" : "s"} on{" "}
              <code className="cp-mono">{pendingPreviewsPath(projectId)}</code>.
              Announce only.
            </p>
          </DaemonReadPanel>
        ) : null}
      </ProjectAuthorityPanel>
    </section>
  );
}

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
 * Today — Personal 2.0 L1. Project list plus HITL announce-only for the
 * first daemon Project. No Confirm/Approve. No fake next-action chrome.
 */
export function TodayPage() {
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
    <section data-page="opc-today">
      <PageHeader
        title="Today"
        lede="What needs the Owner on a real Project. Not Home, not an Inbox."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. Vite is not the product origin.
        This page is empty until Project authority exists.
      </HonestyNote>
      <ProjectAuthorityPanel projection={projects} surface="Today">
        <p className="cp-quiet">
          {projects.data?.length ?? 0} Project
          {(projects.data?.length ?? 0) === 1 ? "" : "s"} on{" "}
          <code className="cp-mono">{PROJECT_LIST_PATH}</code>. Open Projects for
          the list. This is not a decision packet.
        </p>
        {projectId ? (
          <DaemonReadPanel
            projection={hitl}
            surface="Today HITL announcements"
            emptyTitle="Today: no pending ApprovalPreview"
            emptyBody="No pending ApprovalPreview for this Project. Chat cannot Approve. Confirm stays on management HTTP."
            region="opc-hitl"
          >
            <table className="cp-table">
              <caption className="cp-quiet">
                GET {pendingPreviewsPath(projectId)} — announce only; no Confirm
              </caption>
              <thead>
                <tr>
                  <th>Preview</th>
                  <th>Kind</th>
                  <th>Status</th>
                </tr>
              </thead>
              <tbody>
                {(hitl.data ?? []).map((row) => (
                  <tr key={row.previewId} data-row-key={row.previewId}>
                    <td>
                      <code className="cp-mono">{row.previewId}</code>
                    </td>
                    <td>{row.subjectKind}</td>
                    <td>{row.status}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </DaemonReadPanel>
        ) : null}
      </ProjectAuthorityPanel>
    </section>
  );
}

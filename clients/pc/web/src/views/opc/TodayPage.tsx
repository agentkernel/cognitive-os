import { useCallback, useEffect } from "react";
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
 * Today — Personal 2.0 L1. Project list plus HITL announce-only with a
 * deep link into the Projects canvas. No Confirm/Approve. No Inbox L1.
 */
export function TodayPage() {
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
            <HitlCanvasTable
              projectId={projectId}
              rows={hitl.data ?? []}
              deepLink
            />
          </DaemonReadPanel>
        ) : null}
      </ProjectAuthorityPanel>
    </section>
  );
}

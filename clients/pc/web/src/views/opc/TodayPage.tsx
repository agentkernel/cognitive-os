import { useCallback, useEffect } from "react";
import { PageHeader } from "../../components/PageHeader";
import { fetchProjection } from "../../data/fetchProjection";
import {
  PROJECTS_KEY,
  PROJECT_LIST_PATH,
  projectProjectList,
  type ProjectListRow,
} from "../../data/projections/projects";
import { appProjections } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { ProjectAuthorityPanel } from "./ProjectAuthorityPanel";

/**
 * Today — Personal 2.0 L1. First slice: Project-authority honesty only. No
 * attention packets, swimlanes, or fake next-action chrome.
 */
export function TodayPage() {
  const projects = useProjection<ProjectListRow[]>(PROJECTS_KEY);
  const refresh = useCallback(async () => {
    await fetchProjection(
      appProjections,
      PROJECTS_KEY,
      PROJECT_LIST_PATH,
      "management",
      projectProjectList,
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
        Product origin is daemon-served hash /ui/. This page is empty until
        Project authority exists.
      </HonestyNote>
      <ProjectAuthorityPanel projection={projects} surface="Today">
        <p className="cp-quiet">
          {projects.data?.length ?? 0} Project
          {(projects.data?.length ?? 0) === 1 ? "" : "s"} on{" "}
          <code className="cp-mono">{PROJECT_LIST_PATH}</code>. Open Projects for
          the list. This is not a decision packet.
        </p>
      </ProjectAuthorityPanel>
    </section>
  );
}

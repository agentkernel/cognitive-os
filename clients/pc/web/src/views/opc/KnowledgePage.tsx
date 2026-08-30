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
 * Knowledge — L1. First slice is Project-gated honesty. Vault files are not
 * Project authority (P11-T10). No ingest button here.
 */
export function KnowledgePage() {
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
    <section data-page="opc-knowledge">
      <PageHeader
        title="Knowledge"
        lede="Project-scoped knowledge. Files are not Project authority."
      />
      <HonestyNote>
        Markdown Vault import exists on management HTTP. This page does not
        ingest, search, or pretend a file is a Charter.
      </HonestyNote>
      <ProjectAuthorityPanel projection={projects} surface="Knowledge">
        <p className="cp-quiet">
          {projects.data?.length ?? 0} Project
          {(projects.data?.length ?? 0) === 1 ? "" : "s"} in scope. Vault index
          is T10; this slice does not open it.
        </p>
      </ProjectAuthorityPanel>
    </section>
  );
}

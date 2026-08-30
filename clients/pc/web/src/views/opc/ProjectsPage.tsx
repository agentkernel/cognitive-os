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
import { ProjectAuthorityPanel } from "./ProjectAuthorityPanel";

/**
 * Projects — L1 inventory of daemon Project rows. No create/activate control
 * in this slice; confirm-before-activate stays on management HTTP.
 */
export function ProjectsPage() {
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
    <section data-page="opc-projects">
      <PageHeader
        title="Projects"
        lede="Daemon Project aggregate. Not a renamed Task list."
      />
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
      </ProjectAuthorityPanel>
    </section>
  );
}

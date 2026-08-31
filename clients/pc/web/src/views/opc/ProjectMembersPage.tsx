import { useCallback, useEffect } from "react";
import { Link, useParams } from "react-router-dom";
import { PageHeader } from "../../components/PageHeader";
import {
  PROJECT_ROSTER_PATH,
  projectRosterKey,
  type ProjectRosterRow,
} from "../../data/projections/projectWork";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { loadProjectRoster } from "./loadOpcReads";
import { ProjectWorkNav } from "./ProjectWorkNav";

/**
 * Members — daemon GET roster. Select-then-configure via member-config.
 * Add member is Intent. No Install store. No member budget.
 */
export function ProjectMembersPage() {
  const { projectId = "" } = useParams();
  const roster = useProjection<ProjectRosterRow[]>(projectRosterKey(projectId));
  const refresh = useCallback(async () => {
    if (projectId.length === 0) {
      return;
    }
    await loadProjectRoster(projectId);
  }, [projectId]);
  useEffect(() => {
    void refresh();
  }, [refresh]);
  const addHref = projectId ? `/projects/${encodeURIComponent(projectId)}/members/new` : "/projects";

  return (
    <section data-page="opc-project-members">
      <PageHeader
        title="Project members"
        lede="Employee roster for this Project. Select a row to configure. Not Installed Agents."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. GET {PROJECT_ROSTER_PATH} is
        the list. Add member writes join as management Intent. This page does
        not Install and does not mint a seat locally. Role is not merged into
        Agent. Member-level budget is not chrome.
      </HonestyNote>
      <p className="cp-quiet">
        <Link to="/projects">Projects list</Link>
        {projectId ? (
          <>
            {" "}
            · <code className="cp-mono">{projectId}</code>
          </>
        ) : null}
        {" · "}
        <Link to={addHref} className="cp-button">
          Add member
        </Link>
      </p>
      {projectId ? <ProjectWorkNav projectId={projectId} /> : null}
      <DaemonReadPanel
        projection={roster}
        surface="Project members"
        emptyTitle="Project members: empty roster"
        emptyBody="authority_note empty-roster. That is not a missing Team space. No member is seated. Add member still requires PlanRevision slots. This page does not offer Install."
        region="opc-project-roster"
      >
        <table className="cp-table">
          <caption className="cp-quiet">GET {PROJECT_ROSTER_PATH}</caption>
          <thead>
            <tr>
              <th>Employee</th>
              <th>State</th>
              <th>Current manager</th>
            </tr>
          </thead>
          <tbody>
            {(roster.data ?? []).map((row) => (
              <tr key={row.employeeId} data-row-key={row.employeeId}>
                <td>
                  <Link
                    className="cp-button"
                    to={`/projects/${encodeURIComponent(projectId)}/members/${encodeURIComponent(row.employeeId)}`}
                  >
                    {row.employeeId}
                  </Link>
                </td>
                <td>{row.state}</td>
                <td>{row.isCurrentManager}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </DaemonReadPanel>
    </section>
  );
}

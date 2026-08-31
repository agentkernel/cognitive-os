import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { PageHeader } from "../../components/PageHeader";
import { EmptyState } from "../../components/states";
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
 * Members — daemon GET roster. Select-then-view only.
 * Eight tabs and add-member are P12-T04. No Install store.
 */
export function ProjectMembersPage() {
  const { projectId = "" } = useParams();
  const roster = useProjection<ProjectRosterRow[]>(projectRosterKey(projectId));
  const [selectedId, setSelectedId] = useState<string | undefined>();
  const refresh = useCallback(async () => {
    if (projectId.length === 0) {
      return;
    }
    await loadProjectRoster(projectId);
  }, [projectId]);
  useEffect(() => {
    void refresh();
    setSelectedId(undefined);
  }, [refresh]);
  const selected = (roster.data ?? []).find((row) => row.employeeId === selectedId);

  return (
    <section data-page="opc-project-members">
      <PageHeader
        title="Project members"
        lede="Employee roster for this Project. Not Installed Agents."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. GET {PROJECT_ROSTER_PATH} is
        the list. Click a row to view identity. Eight-tab configuration and add
        member are a later card. This page does not Install and does not mint
        a seat. Role is not merged into Agent.
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
        projection={roster}
        surface="Project members"
        emptyTitle="Project members: empty roster"
        emptyBody="authority_note empty-roster. That is not a missing Team space. No member is seated. This page does not offer Install or a fake Add control."
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
                  <button
                    type="button"
                    className="cp-button"
                    aria-pressed={selectedId === row.employeeId}
                    onClick={() => setSelectedId(row.employeeId)}
                  >
                    {row.employeeId}
                  </button>
                </td>
                <td>{row.state}</td>
                <td>{row.isCurrentManager}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {selected ? (
          <table className="cp-table" data-region="opc-member-selected">
            <caption className="cp-quiet">Selected Employee — identity only</caption>
            <tbody>
              <tr>
                <td>Employee</td>
                <td>
                  <code className="cp-mono">{selected.employeeId}</code>
                </td>
              </tr>
              <tr>
                <td>State</td>
                <td>{selected.state}</td>
              </tr>
              <tr>
                <td>Model bound</td>
                <td>{selected.modelBound}</td>
              </tr>
              <tr>
                <td>Runtime binding</td>
                <td>
                  <code className="cp-mono">{selected.runtimeBindingRef}</code>
                </td>
              </tr>
            </tbody>
          </table>
        ) : (
          <EmptyState title="Project members: no member selected">
            Pick a row. This page does not default the first Employee. Eight-tab
            configuration is not this card.
          </EmptyState>
        )}
      </DaemonReadPanel>
    </section>
  );
}

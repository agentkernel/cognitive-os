import { useCallback, useEffect } from "react";
import { Link, useParams } from "react-router-dom";
import { PageHeader } from "../../components/PageHeader";
import {
  PROJECT_AXIS_PATH,
  projectAxisKey,
  type ProjectAxisStageRow,
} from "../../data/projections/projectWork";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { loadProjectAxis } from "./loadOpcReads";
import { ProjectWorkNav } from "./ProjectWorkNav";

/**
 * Runs — current PlanRevision axis (P11-T03/T08 authority).
 * Not a renamed Work timeline. Occurrence ledger needs a routine_id this
 * page does not invent.
 */
export function ProjectRunsPage() {
  const { projectId = "" } = useParams();
  const axis = useProjection<ProjectAxisStageRow[]>(projectAxisKey(projectId));
  const refresh = useCallback(async () => {
    if (projectId.length === 0) {
      return;
    }
    await loadProjectAxis(projectId);
  }, [projectId]);
  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <section data-page="opc-project-runs">
      <PageHeader
        title="Project runs"
        lede="Current PlanRevision axis. Not a renamed Work timeline."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. GET {PROJECT_AXIS_PATH} is
        the run view. Routine occurrence ledger requires a routine_id this page
        does not invent. No Start or Approve control lives here.
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
        projection={axis}
        surface="Project runs"
        emptyTitle="Project runs: no PlanRevision axis"
        emptyBody="No daemon stages. This is not an empty Work list and not a fake run. Occurrence ledger is not inferred."
        region="opc-project-runs"
      >
        <table className="cp-table">
          <caption className="cp-quiet">GET {PROJECT_AXIS_PATH}</caption>
          <thead>
            <tr>
              <th>Position</th>
              <th>Stage</th>
              <th>Confirm</th>
              <th>Ready</th>
              <th>Seated</th>
              <th>Gaps</th>
            </tr>
          </thead>
          <tbody>
            {(axis.data ?? []).map((stage) => (
              <tr key={stage.stageId} data-row-key={stage.stageId}>
                <td>{stage.position}</td>
                <td>
                  <code className="cp-mono">{stage.stageId}</code> {stage.title}
                </td>
                <td>{stage.confirmStatus}</td>
                <td>{stage.ready}</td>
                <td>{stage.seated}</td>
                <td>{stage.gapCount}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </DaemonReadPanel>
    </section>
  );
}

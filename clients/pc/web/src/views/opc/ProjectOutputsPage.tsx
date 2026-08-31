import { useCallback, useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { PageHeader } from "../../components/PageHeader";
import { EmptyState } from "../../components/states";
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
 * Outputs — contracted outputs from the PlanRevision axis.
 * Knowledge files are not Project authority. Select-then-view.
 */
export function ProjectOutputsPage() {
  const { projectId = "" } = useParams();
  const axis = useProjection<ProjectAxisStageRow[]>(projectAxisKey(projectId));
  const [selectedId, setSelectedId] = useState<string | undefined>();
  const refresh = useCallback(async () => {
    if (projectId.length === 0) {
      return;
    }
    await loadProjectAxis(projectId);
  }, [projectId]);
  useEffect(() => {
    void refresh();
    setSelectedId(undefined);
  }, [refresh]);
  const selected = (axis.data ?? []).find((row) => row.stageId === selectedId);

  return (
    <section data-page="opc-project-outputs">
      <PageHeader
        title="Project outputs"
        lede="Contracted stage outputs. Files are not Project authority."
      />
      <HonestyNote>
        Product origin is daemon-served hash /ui/. GET {PROJECT_AXIS_PATH}
        output_contract is the source. Knowledge Vault files are not this list.
        Unknown deliverable type stays unknown. This page does not publish.
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
        surface="Project outputs"
        emptyTitle="Project outputs: no contracted outputs"
        emptyBody="No PlanRevision stages. This is not a Knowledge file list and not a fake publish queue."
        region="opc-project-outputs"
      >
        <table className="cp-table">
          <caption className="cp-quiet">GET {PROJECT_AXIS_PATH} output_contract</caption>
          <thead>
            <tr>
              <th>Stage</th>
              <th>Deliverable</th>
              <th>Save format</th>
            </tr>
          </thead>
          <tbody>
            {(axis.data ?? []).map((stage) => (
              <tr key={stage.stageId} data-row-key={stage.stageId}>
                <td>
                  <button
                    type="button"
                    className="cp-button"
                    aria-pressed={selectedId === stage.stageId}
                    onClick={() => setSelectedId(stage.stageId)}
                  >
                    {stage.title}
                  </button>
                </td>
                <td>{stage.deliverableType}</td>
                <td>{stage.saveFormat}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {selected ? (
          <table className="cp-table" data-region="opc-output-selected">
            <caption className="cp-quiet">Selected contracted output</caption>
            <tbody>
              <tr>
                <td>Stage</td>
                <td>
                  <code className="cp-mono">{selected.stageId}</code>
                </td>
              </tr>
              <tr>
                <td>Digest</td>
                <td>
                  <code className="cp-mono">{selected.outputDigest}</code>
                </td>
              </tr>
              <tr>
                <td>Deliverable</td>
                <td>{selected.deliverableType}</td>
              </tr>
              <tr>
                <td>Open with</td>
                <td>{selected.openWith}</td>
              </tr>
            </tbody>
          </table>
        ) : (
          <EmptyState title="Project outputs: no output selected">
            Pick a contracted stage. Knowledge files are not Project authority.
          </EmptyState>
        )}
      </DaemonReadPanel>
    </section>
  );
}

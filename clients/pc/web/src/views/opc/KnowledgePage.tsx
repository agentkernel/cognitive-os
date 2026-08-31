import { useCallback, useEffect } from "react";
import { PageHeader } from "../../components/PageHeader";
import { fetchProjection } from "../../data/fetchProjection";
import { PROJECTS_KEY, type ProjectListRow } from "../../data/projections/projects";
import { projectResourceList, type ResourceEnvelope } from "../../data/projections/resources";
import { projectVaultIndex, VAULT_INDEX_KEY, vaultIndexPath, type VaultIndexEntry } from "../../data/projections/vault";
import { appProjections } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { loadProjectList, readyProjectId } from "./loadOpcReads";
import { ProjectAuthorityPanel } from "./ProjectAuthorityPanel";

export const OPC_MEMORY_KEY = "opc:memory";
export const OPC_MEMORY_PATH = "/management/resource/v1/list?family=memory";

function projectOpcMemoryRows(body: unknown): ResourceEnvelope[] {
  return projectResourceList(body).resources.filter(
    (row) => row.id.length > 0 && row.id !== "unknown",
  );
}

/**
 * Knowledge — L1. Project-gated Vault index + Memory envelope reads.
 * Files are not Project authority. No ingest / apply-authority / remember.
 */
export function KnowledgePage() {
  const projects = useProjection<ProjectListRow[]>(PROJECTS_KEY);
  const vault = useProjection<VaultIndexEntry[]>(VAULT_INDEX_KEY);
  const memory = useProjection<ResourceEnvelope[]>(OPC_MEMORY_KEY);
  const projectId = readyProjectId(projects);
  const refresh = useCallback(async () => {
    const list = await loadProjectList();
    const id = readyProjectId(list);
    if (!id) {
      return;
    }
    await Promise.all([
      fetchProjection(
        appProjections,
        VAULT_INDEX_KEY,
        vaultIndexPath(id),
        "management",
        projectVaultIndex,
      ),
      fetchProjection(
        appProjections,
        OPC_MEMORY_KEY,
        OPC_MEMORY_PATH,
        "management",
        projectOpcMemoryRows,
      ),
    ]);
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
        Product origin is daemon-served hash /ui/. Vite is not the product origin.
        Markdown Vault import exists on management HTTP. This page does not
        ingest, search, or pretend a file is a Charter.
      </HonestyNote>
      <ProjectAuthorityPanel projection={projects} surface="Knowledge">
        {projectId ? (
          <>
            <DaemonReadPanel
              projection={vault}
              surface="Knowledge Vault index"
              emptyTitle="Knowledge: no Vault index entries"
              emptyBody="The daemon reports no Vault index entries for this Project. Files are not Project authority."
              region="opc-vault"
            >
              <table className="cp-table">
                <caption className="cp-quiet">GET {vaultIndexPath(projectId)}</caption>
                <thead>
                  <tr>
                    <th>Entry</th>
                    <th>Layer</th>
                    <th>Excerpt</th>
                  </tr>
                </thead>
                <tbody>
                  {(vault.data ?? []).map((row) => (
                    <tr key={row.entryId} data-row-key={row.entryId}>
                      <td>
                        <code className="cp-mono">{row.entryId}</code>
                      </td>
                      <td>{row.layer}</td>
                      <td>{row.excerpt}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </DaemonReadPanel>
            <DaemonReadPanel
              projection={memory}
              surface="Knowledge Memory envelope"
              emptyTitle="Knowledge: no admitted Memory objects"
              emptyBody="The Resource Manager memory list is empty. Forget/remember stay on management HTTP. Memory is not the Vault index."
              region="opc-memory"
            >
              <table className="cp-table">
                <caption className="cp-quiet">
                  GET {OPC_MEMORY_PATH}. Tombstones are not in this list.
                </caption>
                <thead>
                  <tr>
                    <th>Memory</th>
                    <th>Health</th>
                  </tr>
                </thead>
                <tbody>
                  {(memory.data ?? []).map((row) => (
                    <tr key={row.id} data-row-key={row.id}>
                      <td>
                        <code className="cp-mono">{row.id}</code>
                      </td>
                      <td>{row.health ?? "unknown"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </DaemonReadPanel>
          </>
        ) : (
          <p className="cp-quiet">
            {projects.data?.length ?? 0} Project
            {(projects.data?.length ?? 0) === 1 ? "" : "s"} in scope. Vault index
            is T10; this slice does not open it without a Project id.
          </p>
        )}
      </ProjectAuthorityPanel>
    </section>
  );
}

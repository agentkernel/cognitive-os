import { useCallback, useEffect, useState, type FormEvent } from "react";
import { readJson } from "../../api";
import { PageHeader } from "../../components/PageHeader";
import { fetchProjection } from "../../data/fetchProjection";
import { containsSecretMaterial } from "../../policy";
import { PROJECTS_KEY, type ProjectListRow } from "../../data/projections/projects";
import { projectResourceList, type ResourceEnvelope } from "../../data/projections/resources";
import {
  projectVaultConflicts,
  projectVaultIndex,
  projectVaultInjectOrder,
  VAULT_CONFLICTS_KEY,
  VAULT_IMPORT_PATH,
  VAULT_INDEX_KEY,
  VAULT_INJECT_ORDER_KEY,
  VAULT_REBUILD_PATH,
  VAULT_RIGHTS_CLASSES,
  vaultConflictsPath,
  vaultImportIsAuthority,
  vaultIndexPath,
  type VaultConflictRow,
  type VaultIndexEntry,
} from "../../data/projections/vault";
import { appProjections } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { httpErrorMessage } from "./httpError";
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
 * Knowledge — ingest + Why this fragment (P12-T07) on daemon `/ui/`.
 * Files are not Project authority. No apply-authority. No Obsidian. No host FS.
 */
export function KnowledgePage() {
  const projects = useProjection<ProjectListRow[]>(PROJECTS_KEY);
  const vault = useProjection<VaultIndexEntry[]>(VAULT_INDEX_KEY);
  const injectOrder = useProjection<string[]>(VAULT_INJECT_ORDER_KEY);
  const conflicts = useProjection<VaultConflictRow[]>(VAULT_CONFLICTS_KEY);
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
        VAULT_INJECT_ORDER_KEY,
        vaultIndexPath(id),
        "management",
        projectVaultInjectOrder,
      ),
      fetchProjection(
        appProjections,
        VAULT_CONFLICTS_KEY,
        vaultConflictsPath(id),
        "management",
        projectVaultConflicts,
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
        Ingest is POST {VAULT_IMPORT_PATH} (owner-paste). Import failure keeps the
        original fields. Files are not a Charter. Host filesystem E2E is not-run.
      </HonestyNote>
      <ProjectAuthorityPanel projection={projects} surface="Knowledge">
        {projectId ? (
          <>
            <VaultIngestForm projectId={projectId} onImported={refresh} />
            <DaemonReadPanel
              projection={injectOrder}
              surface="Why this fragment inject order"
              emptyTitle="Knowledge: inject_order absent"
              emptyBody="vault.index did not return inject_order. Why this fragment does not invent Task-contract or fixed-decision layers."
              region="opc-why-fragment-order"
            >
              <div>
                <p className="cp-quiet">GET {vaultIndexPath(projectId)} inject_order</p>
                <ol data-region="opc-why-fragment-order-list">
                  {(injectOrder.data ?? []).map((layer) => (
                    <li key={layer}>
                      <code className="cp-mono">{layer}</code>
                    </li>
                  ))}
                </ol>
              </div>
            </DaemonReadPanel>
            <DaemonReadPanel
              projection={vault}
              surface="Why this fragment"
              emptyTitle="Knowledge: no Why this fragment excerpts"
              emptyBody="The daemon reports no Vault index excerpts for this Project. Why this fragment does not invent sourced text."
              region="opc-why-fragment"
            >
              <table className="cp-table">
                <caption className="cp-quiet">
                  GET {vaultIndexPath(projectId)} — Why this fragment. Files are
                  not Project authority.
                </caption>
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
                    <tr key={`vault-${row.entryId}`} data-row-key={row.entryId}>
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
              projection={conflicts}
              surface="Knowledge Vault conflicts"
              emptyTitle="Knowledge: no Vault conflicts"
              emptyBody="No conflict records. Last-write-wins without a conflict record is rejected by the daemon."
              region="opc-vault-conflicts"
            >
              <table className="cp-table">
                <caption className="cp-quiet">GET {vaultConflictsPath(projectId)}</caption>
                <thead>
                  <tr>
                    <th>Conflict</th>
                    <th>Path</th>
                    <th>Resolution</th>
                  </tr>
                </thead>
                <tbody>
                  {(conflicts.data ?? []).map((row) => (
                    <tr key={row.conflictId} data-row-key={row.conflictId}>
                      <td>
                        <code className="cp-mono">{row.conflictId}</code>
                      </td>
                      <td>{row.relativePath}</td>
                      <td>{row.resolution}</td>
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
            {(projects.data?.length ?? 0) === 1 ? "" : "s"} in scope. Vault ingest
            is T10; this slice does not open it without a Project id.
          </p>
        )}
      </ProjectAuthorityPanel>
    </section>
  );
}

function VaultIngestForm({
  projectId,
  onImported,
}: {
  projectId: string;
  onImported: () => Promise<void>;
}) {
  const [relativePath, setRelativePath] = useState("notes/note.md");
  const [rightsClass, setRightsClass] = useState<(typeof VAULT_RIGHTS_CLASSES)[number]>("owner-owned");
  const [sourceUri, setSourceUri] = useState("owner-paste");
  const [body, setBody] = useState("");
  const [recordConflict, setRecordConflict] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [receipt, setReceipt] = useState<string | undefined>();

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    const path = relativePath.trim();
    const provenanceUri = sourceUri.trim();
    const markdown = body;
    if (path.length === 0 || provenanceUri.length === 0) {
      setError("Import needs a relative path and provenance. The original fields stay.");
      return;
    }
    if (containsSecretMaterial(path) || containsSecretMaterial(provenanceUri) || containsSecretMaterial(markdown)) {
      setError(
        "Secret-shaped text is not imported. The original fields stay. Secret ingestion is not a Knowledge action.",
      );
      return;
    }
    setBusy(true);
    setError(undefined);
    setReceipt(undefined);
    try {
      const payload: Record<string, unknown> = {
        project_id: projectId,
        relative_path: path,
        rights_class: rightsClass,
        provenance: { source_uri: provenanceUri },
        source_kind: "owner-paste",
        body: markdown,
      };
      if (recordConflict) {
        payload.conflict_policy = "record";
      }
      const written = await readJson(VAULT_IMPORT_PATH, "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(payload),
      });
      if (!written.ok) {
        setError(httpErrorMessage(written.status, written.body));
        return;
      }
      if (vaultImportIsAuthority(written.body)) {
        setError(
          "Daemon claimed is_authority on a Vault file. Files are not Project authority. The original fields stay.",
        );
        return;
      }
      const rebuilt = await readJson(VAULT_REBUILD_PATH, "management", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ project_id: projectId }),
      });
      if (!rebuilt.ok) {
        setError(
          `${httpErrorMessage(rebuilt.status, rebuilt.body)} Import stored the document; index rebuild failed. The original fields stay.`,
        );
        return;
      }
      const record =
        written.body && typeof written.body === "object"
          ? (written.body as Record<string, unknown>)
          : {};
      const documentId = typeof record.document_id === "string" ? record.document_id : "ok";
      setReceipt(documentId);
      await onImported();
    } finally {
      setBusy(false);
    }
  }

  return (
    <form data-region="opc-vault-ingest" className="cp-stack" onSubmit={onSubmit}>
      <h3>Import to Vault</h3>
      <p className="cp-quiet">
        Owner-paste only. Host filesystem E2E is not-run. Files are not Project
        authority. Last-write-wins without a conflict record is rejected.
      </p>
      <label>
        Relative path
        <input
          name="relative_path"
          value={relativePath}
          onChange={(event) => setRelativePath(event.target.value)}
          autoComplete="off"
        />
      </label>
      <label>
        Rights class
        <select
          name="rights_class"
          value={rightsClass}
          onChange={(event) =>
            setRightsClass(event.target.value as (typeof VAULT_RIGHTS_CLASSES)[number])
          }
        >
          {VAULT_RIGHTS_CLASSES.map((item) => (
            <option key={item} value={item}>
              {item}
            </option>
          ))}
        </select>
      </label>
      <label>
        Provenance source_uri
        <input
          name="source_uri"
          value={sourceUri}
          onChange={(event) => setSourceUri(event.target.value)}
          autoComplete="off"
        />
      </label>
      <label>
        Markdown body
        <textarea name="vault-body" value={body} onChange={(event) => setBody(event.target.value)} rows={8} />
      </label>
      <label>
        <input
          type="checkbox"
          name="record_conflict"
          checked={recordConflict}
          onChange={(event) => setRecordConflict(event.target.checked)}
        />{" "}
        Record conflict if this path already exists
      </label>
      <button type="submit" disabled={busy}>
        Import to Vault
      </button>
      {error ? <p data-ingest-error="true">{error}</p> : null}
      {receipt ? (
        <p data-ingest-receipt={receipt} className="cp-quiet">
          Stored document <code className="cp-mono">{receipt}</code>. is_authority
          remains false. host_fs_e2e is not-run.
        </p>
      ) : null}
    </form>
  );
}

import { useCallback, useEffect, useState, type FormEvent } from "react";
import { readJson } from "../../api";
import { ConfirmSurface } from "../../components/ConfirmSurface";
import { PageHeader } from "../../components/PageHeader";
import { fetchProjection } from "../../data/fetchProjection";
import { asRecord } from "../../data/projections";
import {
  CHAT_AUTO_ADMIT_REQUIRES_BACKEND,
  MEMORY_CORRECT_PATH,
  MEMORY_FORGET_PATH,
  MEMORY_PROMOTE_CONFIRM_PATH,
  MEMORY_PROMOTE_REQUEST_PATH,
  memoryPromotesPath,
  projectMemoryPromotes,
  type MemoryPromoteView,
} from "../../data/projections/knowledgeMemory";
import {
  memoryInspectPath,
  memoryObjectPath,
  MEMORY_FORGET_CONSEQUENCE,
  projectMemoryExplain,
} from "../../data/projections/memory";
import { PROJECTS_KEY, type ProjectListRow } from "../../data/projections/projects";
import { projectRosterPath } from "../../data/projections/projectWork";
import { projectResourceList, type ResourceEnvelope } from "../../data/projections/resources";
import {
  projectVaultConflicts,
  projectVaultDocuments,
  projectVaultIndex,
  projectVaultInjectOrder,
  projectVaultLabeled,
  VAULT_CONFLICTS_KEY,
  VAULT_IMPORT_PATH,
  VAULT_INDEX_KEY,
  VAULT_INJECT_ORDER_KEY,
  VAULT_REBUILD_PATH,
  VAULT_RIGHTS_CLASSES,
  vaultConflictsPath,
  vaultDocumentsPath,
  vaultImportIsAuthority,
  vaultIndexPath,
  vaultLabeledPath,
  type VaultConflictRow,
  type VaultDocumentStatus,
  type VaultIndexEntry,
  type VaultLabeledEntry,
} from "../../data/projections/vault";
import { appProjections, type Projection } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { containsSecretMaterial } from "../../policy";
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

function jsonProjection<T>(
  result: { ok: boolean; status: number; body: unknown },
  source: string,
  map: (body: unknown) => T[],
): Projection<T[]> {
  if (result.status === 403) {
    return {
      status: "denied",
      source,
      error: { code: "denied", httpStatus: 403, message: "session denied" },
    };
  }
  if (!result.ok) {
    return {
      status: "unknown",
      source,
      error: {
        code: `HTTP_${result.status}`,
        httpStatus: result.status,
        message: "labeled or document read failed",
      },
    };
  }
  const data = map(result.body);
  return { status: data.length === 0 ? "empty" : "ready", data, source };
}

/**
 * Knowledge — ingest + Why this fragment (P12-T07) + P13-T07 labels / Memory
 * authority on daemon `/ui/`. Files are not Project authority. No Obsidian.
 */
export function KnowledgePage() {
  const projects = useProjection<ProjectListRow[]>(PROJECTS_KEY);
  const vault = useProjection<VaultIndexEntry[]>(VAULT_INDEX_KEY);
  const injectOrder = useProjection<string[]>(VAULT_INJECT_ORDER_KEY);
  const conflicts = useProjection<VaultConflictRow[]>(VAULT_CONFLICTS_KEY);
  const memory = useProjection<ResourceEnvelope[]>(OPC_MEMORY_KEY);
  const [labeled, setLabeled] = useState<Projection<VaultLabeledEntry[]>>({ status: "loading" });
  const [documents, setDocuments] = useState<Projection<VaultDocumentStatus[]>>({
    status: "loading",
  });
  const [promotes, setPromotes] = useState<MemoryPromoteView[]>([]);
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
    const [labeledRead, documentRead, promoteRead] = await Promise.all([
      readJson(vaultLabeledPath(id), "management"),
      readJson(vaultDocumentsPath(id), "management"),
      readJson(memoryPromotesPath(id), "management"),
    ]);
    setLabeled(jsonProjection(labeledRead, vaultLabeledPath(id), projectVaultLabeled));
    setDocuments(jsonProjection(documentRead, vaultDocumentsPath(id), projectVaultDocuments));
    setPromotes(promoteRead.ok ? projectMemoryPromotes(promoteRead.body) : []);
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
        original fields. Labeled fragments come from GET vault.labeled. Memory
        inspect / correct / promote / forget stay on management HTTP. Chat
        auto-admission is honest-empty / Requires-backend (T06 turns are not
        listed as admit candidates; 0 Admit buttons). Files are not a Charter.
        Host filesystem E2E is not-run until P13-T13.
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
              projection={labeled}
              surface="Knowledge fragment labels"
              emptyTitle="Knowledge: no labeled fragments"
              emptyBody="vault.labeled returned no excerpts. Labels are not invented. Files are not Project authority."
              region="opc-knowledge-labels"
            >
              <table className="cp-table">
                <caption className="cp-quiet">
                  GET {vaultLabeledPath(projectId)} —
                  provenance / rights / freshness / exclusion. is_authority stays false.
                </caption>
                <thead>
                  <tr>
                    <th>Path</th>
                    <th>Provenance</th>
                    <th>Rights</th>
                    <th>Freshness</th>
                    <th>Exclusion</th>
                    <th>Untrusted</th>
                    <th>Excerpt</th>
                  </tr>
                </thead>
                <tbody>
                  {(labeled.data ?? []).map((row) => (
                    <tr key={row.entryId} data-row-key={`label-${row.entryId}`}>
                      <td>
                        <code className="cp-mono">{row.relativePath}</code>
                      </td>
                      <td>{row.provenanceSourceUri}</td>
                      <td>{row.rightsClass}</td>
                      <td>{row.freshness}</td>
                      <td>
                        {row.exclusion}
                        {row.exclusionReason ? ` (${row.exclusionReason})` : ""}
                      </td>
                      <td>{row.untrustedObservation ? "yes" : "no"}</td>
                      <td>{row.excerpt}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </DaemonReadPanel>
            <DaemonReadPanel
              projection={documents}
              surface="Knowledge document status"
              emptyTitle="Knowledge: no stored Vault documents"
              emptyBody="No stored documents. A failed rebuild still leaves an imported original visible as not-indexed."
              region="opc-knowledge-documents"
            >
              <table className="cp-table">
                <caption className="cp-quiet">
                  GET {vaultDocumentsPath(projectId)} — original remains visible
                  when index_status is not-indexed.
                </caption>
                <thead>
                  <tr>
                    <th>Document</th>
                    <th>Path</th>
                    <th>Provenance</th>
                    <th>Index</th>
                  </tr>
                </thead>
                <tbody>
                  {(documents.data ?? []).map((row) => (
                    <tr key={row.documentId} data-row-key={`doc-${row.documentId}`}>
                      <td>
                        <code className="cp-mono">{row.documentId}</code>
                      </td>
                      <td>{row.relativePath}</td>
                      <td>{row.provenanceSourceUri}</td>
                      <td>{row.indexStatus}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
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
            <div data-region="opc-knowledge-auto-admit">
              <h3>Chat auto-admission</h3>
              <p className="cp-quiet">{CHAT_AUTO_ADMIT_REQUIRES_BACKEND}</p>
            </div>
            <DaemonReadPanel
              projection={memory}
              surface="Knowledge Memory envelope"
              emptyTitle="Knowledge: no admitted Memory objects"
              emptyBody="The Resource Manager memory list is empty. Inspect / correct / promote / forget stay on management HTTP. Memory is not the Vault index."
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
            <KnowledgeMemoryAuthority
              projectId={projectId}
              projects={projects.data ?? []}
              memories={memory.data ?? []}
              promotes={promotes}
              onChanged={refresh}
            />
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

function KnowledgeMemoryAuthority({
  projectId,
  projects,
  memories,
  promotes,
  onChanged,
}: {
  projectId: string;
  projects: ProjectListRow[];
  memories: ResourceEnvelope[];
  promotes: MemoryPromoteView[];
  onChanged: () => Promise<void>;
}) {
  const [selectedId, setSelectedId] = useState<string | undefined>();
  const [canonical, setCanonical] = useState<string | undefined>();
  const [inspectError, setInspectError] = useState<string | undefined>();
  const [correctText, setCorrectText] = useState("");
  const [employeeId, setEmployeeId] = useState("");
  const [forgetReason, setForgetReason] = useState("");
  const [toProjectId, setToProjectId] = useState("");
  const [toEmployeeId, setToEmployeeId] = useState("");
  const [pending, setPending] = useState<MemoryPromoteView | undefined>();
  const [message, setMessage] = useState<string | undefined>();
  const others = projects.filter((row) => row.projectId !== projectId && row.projectId.length > 0);

  async function inspect(id: string) {
    setSelectedId(id);
    setInspectError(undefined);
    setCanonical(undefined);
    const [inspectRead, objectRead] = await Promise.all([
      readJson(memoryInspectPath(id), "management"),
      readJson(memoryObjectPath(id), "management"),
    ]);
    if (!inspectRead.ok) {
      setInspectError(httpErrorMessage(inspectRead.status, inspectRead.body));
      return;
    }
    const explained = projectMemoryExplain(objectRead.body);
    setCanonical(explained.canonicalJson);
  }

  async function correct() {
    if (!selectedId || correctText.trim().length === 0 || employeeId.trim().length === 0) {
      setMessage("Correct needs a selected Memory, employee_id, and replacement text.");
      return;
    }
    if (containsSecretMaterial(correctText)) {
      setMessage("Secret-shaped text is not corrected into Memory.");
      return;
    }
    const written = await readJson(MEMORY_CORRECT_PATH, "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        memory_id: selectedId,
        project_id: projectId,
        employee_id: employeeId.trim(),
        text: correctText,
      }),
    });
    if (!written.ok) {
      setMessage(httpErrorMessage(written.status, written.body));
      return;
    }
    setMessage("Corrected on management HTTP.");
    await onChanged();
  }

  async function forget() {
    if (!selectedId || forgetReason.trim().length === 0) {
      setMessage("Forget needs a selected Memory and a reason.");
      return;
    }
    const written = await readJson(MEMORY_FORGET_PATH, "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ memory_id: selectedId, reason: forgetReason.trim() }),
    });
    if (!written.ok) {
      setMessage(httpErrorMessage(written.status, written.body));
      return;
    }
    setMessage(`Forgotten ${selectedId}. ${MEMORY_FORGET_CONSEQUENCE}`);
    setSelectedId(undefined);
    await onChanged();
  }

  async function requestPromote() {
    if (!selectedId || toProjectId.trim().length === 0 || toEmployeeId.trim().length === 0) {
      setMessage("Promote preview needs a Memory, target Project, and target employee.");
      return;
    }
    const written = await readJson(MEMORY_PROMOTE_REQUEST_PATH, "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        memory_id: selectedId,
        from_project_id: projectId,
        to_project_id: toProjectId.trim(),
        to_employee_id: toEmployeeId.trim(),
      }),
    });
    if (!written.ok) {
      setMessage(httpErrorMessage(written.status, written.body));
      return;
    }
    const row = asRecord(written.body);
    setPending({
      promoteId: String(row.promote_id ?? ""),
      memoryId: String(row.memory_id ?? selectedId),
      fromProjectId: String(row.from_project_id ?? projectId),
      toProjectId: String(row.to_project_id ?? toProjectId),
      previewDigest: String(row.preview_digest ?? ""),
      status: String(row.status ?? "pending"),
    });
    setMessage("Promote preview pending. The target Project does not yet have a copy.");
    await onChanged();
  }

  async function confirmPromote() {
    if (!pending) {
      return;
    }
    const written = await readJson(MEMORY_PROMOTE_CONFIRM_PATH, "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        promote_id: pending.promoteId,
        preview_digest: pending.previewDigest,
      }),
    });
    if (!written.ok) {
      setMessage(httpErrorMessage(written.status, written.body));
      return;
    }
    setPending(undefined);
    setMessage("Promote confirmed. The copy is a new Memory id.");
    await onChanged();
  }

  return (
    <div data-region="opc-knowledge-memory" className="cp-stack">
      <h3>Memory inspect / correct / promote / forget</h3>
      <p className="cp-quiet">
        Management HTTP only. Tombstones do not resurrect. Cross-Project promote
        needs an Owner preview digest.
      </p>
      <label>
        Select Memory
        <select
          name="memory_id"
          value={selectedId ?? ""}
          onChange={(event) => {
            const id = event.target.value;
            if (id.length > 0) {
              void inspect(id);
            }
          }}
        >
          <option value="">none</option>
          {memories.map((row) => (
            <option key={row.id} value={row.id}>
              {row.id}
            </option>
          ))}
        </select>
      </label>
      {inspectError ? <p data-memory-error="true">{inspectError}</p> : null}
      {canonical ? (
        <pre className="cp-mono" data-memory-canonical="true">
          {canonical}
        </pre>
      ) : null}
      <label>
        Employee id (same Project)
        <input
          name="employee_id"
          value={employeeId}
          onChange={(event) => setEmployeeId(event.target.value)}
          autoComplete="off"
        />
      </label>
      <label>
        Correct text
        <textarea
          name="correct_text"
          value={correctText}
          onChange={(event) => setCorrectText(event.target.value)}
          rows={3}
        />
      </label>
      <button type="button" onClick={() => void correct()}>
        Correct Memory
      </button>
      <label>
        Forget reason
        <input
          name="forget_reason"
          value={forgetReason}
          onChange={(event) => setForgetReason(event.target.value)}
          autoComplete="off"
        />
      </label>
      <button type="button" onClick={() => void forget()}>
        Forget Memory
      </button>
      <label>
        Target Project
        <select
          name="to_project_id"
          value={toProjectId}
          onChange={(event) => {
            setToProjectId(event.target.value);
            void loadTargetEmployee(event.target.value, setToEmployeeId);
          }}
        >
          <option value="">none</option>
          {others.map((row) => (
            <option key={row.projectId} value={row.projectId}>
              {row.titleSummary ?? row.projectId}
            </option>
          ))}
        </select>
      </label>
      <label>
        Target employee id
        <input
          name="to_employee_id"
          value={toEmployeeId}
          onChange={(event) => setToEmployeeId(event.target.value)}
          autoComplete="off"
        />
      </label>
      <button type="button" onClick={() => void requestPromote()}>
        Request promote preview
      </button>
      {pending ? (
        <ConfirmSurface
          title="Owner promote preview"
          consequences="This copies one admitted Memory into another Project after the digest matches. The source Memory is not rewritten."
          targets={[
            `promote_id ${pending.promoteId}`,
            `preview_digest ${pending.previewDigest}`,
            `from ${pending.fromProjectId}`,
            `to ${pending.toProjectId}`,
          ]}
          confirmLabel={`I am binding promote ${pending.promoteId} to digest ${pending.previewDigest}`}
          actionLabel="Promote Memory"
          onConfirm={() => void confirmPromote()}
        />
      ) : null}
      {promotes.length > 0 ? (
        <table className="cp-table">
          <caption className="cp-quiet">GET {memoryPromotesPath(projectId)}</caption>
          <thead>
            <tr>
              <th>Promote</th>
              <th>Status</th>
              <th>Target</th>
            </tr>
          </thead>
          <tbody>
            {promotes.map((row) => (
              <tr key={row.promoteId} data-row-key={`promote-${row.promoteId}`}>
                <td>
                  <code className="cp-mono">{row.promoteId}</code>
                </td>
                <td>{row.status}</td>
                <td>{row.toProjectId}</td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : null}
      {message ? <p data-memory-message="true">{message}</p> : null}
    </div>
  );
}

async function loadTargetEmployee(
  projectId: string,
  setEmployeeId: (value: string) => void,
) {
  if (projectId.length === 0) {
    setEmployeeId("");
    return;
  }
  const roster = await readJson(projectRosterPath(projectId), "management");
  if (!roster.ok) {
    return;
  }
  const record = asRecord(roster.body);
  const rows = Array.isArray(record.roster) ? record.roster : [];
  const first = rows.find((item) => {
    const row = asRecord(item);
    return typeof row.employee_id === "string" && row.employee_id.length > 0;
  });
  if (first) {
    setEmployeeId(String(asRecord(first).employee_id));
  }
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

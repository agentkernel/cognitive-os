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
import { EmptyState } from "../../components/states";
import { HonestyNote } from "../../state/HonestyNote";
import { DaemonReadPanel } from "./DaemonReadPanel";
import { httpErrorMessage } from "./httpError";
import { loadProjectList, readyProjectId } from "./loadOpcReads";
import { ProjectAuthorityPanel } from "./ProjectAuthorityPanel";

export const OPC_MEMORY_KEY = "opc:memory";
export const OPC_MEMORY_PATH = "/management/resource/v1/list?family=memory";

const KNOWLEDGE_TABS = [
  { id: "files", label: "项目资料" },
  { id: "import", label: "导入" },
  { id: "why", label: "为什么用这段" },
  { id: "memory", label: "记忆" },
] as const;

type KnowledgeTab = (typeof KNOWLEDGE_TABS)[number]["id"];
type ImportSource = "files" | "directory" | "link" | "image" | "video" | "typed";
type ImportPolicy = "copy" | "reference";

function safeImportName(name: string): string {
  const trimmed = name.replace(/[\\/]+/g, "-").replace(/^\.+/, "").trim();
  return trimmed.length > 0 ? trimmed : "untitled.md";
}

function catalogKind(row: VaultDocumentStatus): "markdown" | "link" | "image" | "other" {
  const path = row.relativePath.toLowerCase();
  const uri = row.provenanceSourceUri.toLowerCase();
  if (uri.startsWith("http://") || uri.startsWith("https://") || path.includes("/links/")) {
    return "link";
  }
  if (/\.(png|jpe?g|gif|webp|svg)$/i.test(path) || uri.includes("image:")) {
    return "image";
  }
  if (path.endsWith(".md") || path.endsWith(".markdown")) {
    return "markdown";
  }
  return "other";
}

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
 * Knowledge — v9 Owner chrome for files / import / why / memory on daemon `/ui/`.
 * Files are not Project authority. Obsidian is not bundled.
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
  const [tab, setTab] = useState<KnowledgeTab>("files");
  const [fileQuery, setFileQuery] = useState("");
  const [fileKind, setFileKind] = useState<"all" | "markdown" | "link" | "image">("all");
  const projectId = readyProjectId(projects);
  const locked =
    !projectId &&
    (projects.status === "empty" ||
      (projects.status === "ready" && (projects.data?.length ?? 0) === 0));
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
    <section data-page="opc-knowledge" data-knowledge-ia="v9">
      <PageHeader
        title={locked ? "知识已锁定" : "当前项目资料"}
        lede={
          locked
            ? "没有项目时不能进。创建到 ② 流程需要输入时，只为当前草稿打开。"
            : "不必另装笔记应用。解析失败保留原件可重试。离线只读上次索引。"
        }
      />
      <HonestyNote placement="secondary">
        资料不是项目权威。导入走已有 Vault 写入。密钥形态不进知识库。聊天自动入记忆仍是
        Requires-backend。
      </HonestyNote>
      <ProjectAuthorityPanel
        projection={projects}
        surface="知识"
        emptyBody="知识已锁定。没有项目时不能进。创建到 ② 流程需要输入时，只为当前草稿打开。"
        leadHonesty={false}
      >
        {projectId ? (
          <>
            <p className="cp-quiet" role="tablist" aria-label="知识">
              {KNOWLEDGE_TABS.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  role="tab"
                  aria-selected={tab === item.id}
                  className={tab === item.id ? "cp-button cp-button--primary" : "cp-button"}
                  onClick={() => setTab(item.id)}
                >
                  {item.label}
                </button>
              ))}
            </p>
            {tab === "files" ? (
              <KnowledgeFilesTab
                projectId={projectId}
                documents={documents}
                labeled={labeled}
                conflicts={conflicts}
                query={fileQuery}
                kind={fileKind}
                onQuery={setFileQuery}
                onKind={setFileKind}
                onImport={() => setTab("import")}
              />
            ) : null}
            {tab === "import" ? <VaultIngestForm projectId={projectId} onImported={refresh} /> : null}
            {tab === "why" ? (
              <KnowledgeWhyTab
                projectId={projectId}
                injectOrder={injectOrder}
                vault={vault}
              />
            ) : null}
            {tab === "memory" ? (
              <>
                <div data-region="opc-knowledge-auto-admit">
                  <h3>对话自动进入可检查记忆</h3>
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
            ) : null}
          </>
        ) : (
          <p className="cp-quiet" data-region="opc-knowledge-locked">
            知识已锁定。没有项目时不能进。创建到 ② 流程需要输入时，只为当前草稿打开。
          </p>
        )}
      </ProjectAuthorityPanel>
    </section>
  );
}

function KnowledgeFilesTab({
  projectId,
  documents,
  labeled,
  conflicts,
  query,
  kind,
  onQuery,
  onKind,
  onImport,
}: {
  projectId: string;
  documents: Projection<VaultDocumentStatus[]>;
  labeled: Projection<VaultLabeledEntry[]>;
  conflicts: Projection<VaultConflictRow[]>;
  query: string;
  kind: "all" | "markdown" | "link" | "image";
  onQuery: (value: string) => void;
  onKind: (value: "all" | "markdown" | "link" | "image") => void;
  onImport: () => void;
}) {
  const catalog = documents.data ?? [];
  const needle = query.trim().toLowerCase();
  const visible = catalog.filter((row) => {
    if (kind !== "all" && catalogKind(row) !== kind) {
      return false;
    }
    if (needle.length > 0 && !row.relativePath.toLowerCase().includes(needle)) {
      return false;
    }
    return true;
  });
  return (
    <div data-region="opc-knowledge-files" className="cp-stack">
      <h3>资料</h3>
      <p className="cp-quiet">可按资料类型和关键词查看。资料不是项目权威。</p>
      <div className="knowledge-filters">
        <label>
          资料类型
          <select
            name="knowledge-kind"
            value={kind}
            onChange={(event) => onKind(event.target.value as typeof kind)}
          >
            <option value="all">全部类型</option>
            <option value="markdown">Markdown</option>
            <option value="link">链接</option>
            <option value="image">图片</option>
          </select>
        </label>
        <label>
          关键词
          <input
            name="knowledge-query"
            type="search"
            value={query}
            onChange={(event) => onQuery(event.target.value)}
            autoComplete="off"
            placeholder="按标题检索…"
          />
        </label>
      </div>
      {documents.status === "empty" || (documents.status === "ready" && catalog.length === 0) ? (
        <EmptyState
          title="还没资料"
          action={
            <button type="button" className="cp-button cp-button--primary" onClick={onImport}>
              导入资料
            </button>
          }
        >
          用导入把文件、链接或图片元数据放进当前范围。这不是项目权威。
        </EmptyState>
      ) : (
        <DaemonReadPanel
          projection={documents}
          surface="Knowledge document status"
          emptyTitle="Knowledge: no stored Vault documents"
          emptyBody="No stored documents. A failed rebuild still leaves an imported original visible as not-indexed."
          region="opc-knowledge-documents"
        >
          {visible.length === 0 ? (
            <EmptyState title="没有匹配的资料">
              当前类型或关键词没有命中。这不是索引为零。
            </EmptyState>
          ) : (
            <table className="cp-table">
              <caption className="cp-quiet">
                GET {vaultDocumentsPath(projectId)} — original remains visible when
                index_status is not-indexed. is_authority stays false.
              </caption>
              <thead>
                <tr>
                  <th>资料</th>
                  <th>路径</th>
                  <th>类型</th>
                  <th>来源</th>
                  <th>索引</th>
                </tr>
              </thead>
              <tbody>
                {visible.map((row) => (
                  <tr key={row.documentId} data-row-key={`doc-${row.documentId}`}>
                    <td>
                      <code className="cp-mono">{row.documentId}</code>
                    </td>
                    <td>{row.relativePath}</td>
                    <td>{catalogKind(row)}</td>
                    <td>{row.provenanceSourceUri}</td>
                    <td>{row.indexStatus}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </DaemonReadPanel>
      )}
      {catalog.length > 0 ? (
      <DaemonReadPanel
        projection={labeled}
        surface="资料摘录标签"
        emptyTitle="还没有摘录标签"
        emptyBody="没有摘录。标签不编造。资料不是项目权威。"
        region="opc-knowledge-labels"
      >
        <table className="cp-table">
          <caption className="cp-quiet">
            GET {vaultLabeledPath(projectId)}。来源 / 权利 / 新鲜度 / 排除。资料不是项目权威。
          </caption>
          <thead>
            <tr>
              <th>路径</th>
              <th>来源</th>
              <th>权利</th>
              <th>新鲜度</th>
              <th>排除</th>
              <th>不可信</th>
              <th>摘录</th>
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
      ) : null}
      {catalog.length > 0 ? (
      <DaemonReadPanel
        projection={conflicts}
        surface="资料冲突"
        emptyTitle="没有冲突记录"
        emptyBody="没有冲突。无记录的后写覆盖会被拒绝。"
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
    </div>
  );
}

function KnowledgeWhyTab({
  projectId,
  injectOrder,
  vault,
}: {
  projectId: string;
  injectOrder: Projection<string[]>;
  vault: Projection<VaultIndexEntry[]>;
}) {
  return (
    <div data-region="opc-knowledge-why" className="cp-stack">
      <h3>为什么用了这段</h3>
      <p className="cp-quiet">范围、来源、新鲜度、删减损失都要看见。</p>
      <DaemonReadPanel
        projection={injectOrder}
        surface="为什么用了这段 · 注入顺序"
        emptyTitle="知识：注入顺序缺失"
        emptyBody="vault.index 没有返回 inject_order。这里不编造任务合同或固定决策层。"
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
        surface="为什么用了这段"
        emptyTitle="知识：还没有摘录"
        emptyBody="daemon 没有返回本项目的 Vault 摘录。这里不编造来源文字。"
        region="opc-why-fragment"
      >
        <table className="cp-table">
          <caption className="cp-quiet">
            GET {vaultIndexPath(projectId)}。资料不是项目权威。
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
    </div>
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

async function postVaultImport(
  projectId: string,
  payload: Record<string, unknown>,
): Promise<{ ok: true; documentId: string } | { ok: false; error: string; rebuild: boolean }> {
  const written = await readJson(VAULT_IMPORT_PATH, "management", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!written.ok) {
    return { ok: false, error: httpErrorMessage(written.status, written.body), rebuild: false };
  }
  if (vaultImportIsAuthority(written.body)) {
    return {
      ok: false,
      error:
        "Daemon claimed is_authority on a Vault file. Files are not Project authority. The original fields stay.",
      rebuild: false,
    };
  }
  const rebuilt = await readJson(VAULT_REBUILD_PATH, "management", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ project_id: projectId }),
  });
  if (!rebuilt.ok) {
    return {
      ok: false,
      error: `${httpErrorMessage(rebuilt.status, rebuilt.body)} Import stored the document; index rebuild failed. The original fields stay.`,
      rebuild: false,
    };
  }
  const record =
    written.body && typeof written.body === "object" ? (written.body as Record<string, unknown>) : {};
  const documentId = typeof record.document_id === "string" ? record.document_id : "ok";
  return { ok: true, documentId };
}

function readFileAsText(file: File): Promise<string> {
  if (typeof file.text === "function") {
    return file.text();
  }
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result ?? ""));
    reader.onerror = () => reject(reader.error ?? new Error("file read failed"));
    reader.readAsText(file);
  });
}

function metadataMarkdown(file: File, kind: "image" | "video"): string {
  return `# ${file.name}

- kind: ${kind}
- type: ${file.type || "unknown"}
- bytes: ${file.size}

Binary is not stored in Vault. This metadata is not Project authority.
`;
}

function VaultIngestForm({
  projectId,
  onImported,
}: {
  projectId: string;
  onImported: () => Promise<void>;
}) {
  const [source, setSource] = useState<ImportSource>("files");
  const [policy, setPolicy] = useState<ImportPolicy>("copy");
  const [relativePath, setRelativePath] = useState("notes/note.md");
  const [rightsClass, setRightsClass] = useState<(typeof VAULT_RIGHTS_CLASSES)[number]>("owner-owned");
  const [sourceUri, setSourceUri] = useState("owner-paste");
  const [linkUri, setLinkUri] = useState("");
  const [body, setBody] = useState("");
  const [pickedFiles, setPickedFiles] = useState<File[]>([]);
  const [recordConflict, setRecordConflict] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | undefined>();
  const [receipt, setReceipt] = useState<string | undefined>();
  const usesFiles = source === "files" || source === "directory" || source === "image" || source === "video";

  async function importPayload(payload: Record<string, unknown>): Promise<boolean> {
    if (recordConflict) {
      payload.conflict_policy = "record";
    }
    const result = await postVaultImport(projectId, payload);
    if (!result.ok) {
      setError(result.error);
      return false;
    }
    setReceipt(result.documentId);
    return true;
  }

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    setError(undefined);
    setReceipt(undefined);
    setBusy(true);
    try {
      if (usesFiles) {
        const form = event.currentTarget as HTMLFormElement;
        const fileInput = form.querySelector('input[name="vault-files"]') as HTMLInputElement | null;
        const selected = fileInput ? Array.from(fileInput.files ?? []) : pickedFiles;
        if (selected.length === 0) {
          setError("Import needs a file. The original fields stay.");
          return;
        }
        for (const file of selected) {
          const name = safeImportName(file.name);
          if (containsSecretMaterial(name)) {
            setError(
              "Secret-shaped text is not imported. The original fields stay. Secret ingestion is not a Knowledge action.",
            );
            return;
          }
          let markdown: string;
          if (source === "image" || source === "video") {
            markdown = metadataMarkdown(file, source);
          } else {
            try {
              markdown = await readFileAsText(file);
            } catch {
              setError("The file could not be read. The original fields stay.");
              return;
            }
            if (markdown.includes("\u0000")) {
              setError(
                "Binary bytes are not Vault Markdown. The original fields stay. Files are not Project authority.",
              );
              return;
            }
          }
          if (containsSecretMaterial(markdown)) {
            setError(
              "Secret-shaped text is not imported. The original fields stay. Secret ingestion is not a Knowledge action.",
            );
            return;
          }
          const storedBody =
            policy === "reference"
              ? `# reference\n\nfile:${name}\n\nBody was not copied. This is not Project authority.\n`
              : markdown;
          const ok = await importPayload({
            project_id: projectId,
            relative_path: `inbox/${name}${source === "image" || source === "video" ? ".md" : ""}`.replace(
              /\.md\.md$/,
              ".md",
            ),
            rights_class: rightsClass,
            provenance: { source_uri: `file:${name}` },
            source_kind: "markdown-file",
            body: storedBody,
          });
          if (!ok) {
            return;
          }
        }
        await onImported();
        return;
      }
      if (source === "link") {
        const uri = linkUri.trim();
        if (uri.length === 0) {
          setError("Import needs a link. The original fields stay.");
          return;
        }
        if (containsSecretMaterial(uri)) {
          setError(
            "Secret-shaped text is not imported. The original fields stay. Secret ingestion is not a Knowledge action.",
          );
          return;
        }
        const slug = safeImportName(uri.replace(/^https?:\/\//, "").slice(0, 48));
        const markdown =
          policy === "reference"
            ? `# reference\n\n${uri}\n\nBody was not copied. This is not Project authority.\n`
            : `# link\n\n${uri}\n`;
        const ok = await importPayload({
          project_id: projectId,
          relative_path: `links/${slug}.md`,
          rights_class: rightsClass,
          provenance: { source_uri: uri },
          source_kind: "owner-paste",
          body: markdown,
        });
        if (!ok) {
          return;
        }
        await onImported();
        return;
      }
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
      const ok = await importPayload({
        project_id: projectId,
        relative_path: path,
        rights_class: rightsClass,
        provenance: { source_uri: provenanceUri },
        source_kind: "owner-paste",
        body: markdown,
      });
      if (!ok) {
        return;
      }
      await onImported();
    } finally {
      setBusy(false);
    }
  }

  return (
    <form data-region="opc-vault-ingest" className="cp-stack" onSubmit={onSubmit}>
      <h3>导入资料</h3>
      <p className="cp-quiet">
        选范围、复制或引用、来源种类。密钥不得进入 Vault。资料不是项目权威。
      </p>
      <label>
        来源种类
        <select
          name="import_source"
          value={source}
          onChange={(event) => {
            setSource(event.target.value as ImportSource);
            setPickedFiles([]);
          }}
        >
          <option value="files">文件</option>
          <option value="directory">目录</option>
          <option value="link">链接</option>
          <option value="image">图片</option>
          <option value="video">视频元数据</option>
          <option value="typed">手写笔记</option>
        </select>
      </label>
      <label>
        复制或引用
        <select
          name="import_policy"
          value={policy}
          onChange={(event) => setPolicy(event.target.value as ImportPolicy)}
        >
          <option value="copy">复制到 Vault（须有权复用）</option>
          <option value="reference">引用原件（不复制正文）</option>
        </select>
      </label>
      {usesFiles ? (
        <label>
          {source === "directory" ? "目录" : "文件"}
          <input
            name="vault-files"
            type="file"
            multiple={source !== "directory"}
            accept={source === "image" ? "image/*" : source === "video" ? "video/*" : undefined}
            onChange={(event) => setPickedFiles(Array.from(event.target.files ?? []))}
          />
        </label>
      ) : null}
      {source === "link" ? (
        <label>
          链接
          <input
            name="import_link"
            value={linkUri}
            onChange={(event) => setLinkUri(event.target.value)}
            autoComplete="off"
          />
        </label>
      ) : null}
      {source === "typed" ? (
        <>
          <label>
            相对路径
            <input
              name="relative_path"
              value={relativePath}
              onChange={(event) => setRelativePath(event.target.value)}
              autoComplete="off"
            />
          </label>
          <label>
            来源
            <input
              name="source_uri"
              value={sourceUri}
              onChange={(event) => setSourceUri(event.target.value)}
              autoComplete="off"
            />
          </label>
          <label>
            笔记正文
            <textarea
              name="vault-body"
              value={body}
              onChange={(event) => setBody(event.target.value)}
              rows={8}
            />
          </label>
        </>
      ) : null}
      <label>
        权利类别
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
        <input
          type="checkbox"
          name="record_conflict"
          checked={recordConflict}
          onChange={(event) => setRecordConflict(event.target.checked)}
        />{" "}
        Record conflict if this path already exists
      </label>
      <button type="submit" disabled={busy}>
        开始导入
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

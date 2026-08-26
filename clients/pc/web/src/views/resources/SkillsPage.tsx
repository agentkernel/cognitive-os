import { useCallback, useEffect, useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import { readJson } from "../../api";
import { ConfirmSurface } from "../../components/ConfirmSurface";
import { FactGrid } from "../../components/FactGrid";
import { Inspector } from "../../components/Inspector";
import { MasterList } from "../../components/MasterList";
import { PageHeader } from "../../components/PageHeader";
import { ReceiptLine } from "../../components/ReceiptLine";
import { EmptyState } from "../../components/states";
import { fetchProjection } from "../../data/fetchProjection";
import { asRecord } from "../../data/projections";
import {
  envelopeAtBound,
  projectResourceList,
  resourceListKey,
  resourceListPath,
  type ResourceEnvelope,
  type ResourceListView,
} from "../../data/projections/resources";
import {
  SKILL_IMPORT_HONESTY,
  SKILL_PERMISSION_ANNOTATION,
  SKILL_REVOKE_CONSEQUENCE,
  projectSkillExplain,
  skillBindBody,
  skillExplainKey,
  skillExplainPath,
  skillImportBody,
  skillMasterFooter,
  type SkillBindPreview,
  type SkillExplainView,
  type SkillImportPreview,
} from "../../data/projections/skills";
import { appProjections, type Projection } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { useInspectorClear } from "../../shell/useInspectorClear";
import { HonestyNote } from "../../state/HonestyNote";
import { uuidV7 } from "../../taskDraft";
import { ProjectionState } from "../providers/ProjectionState";

/**
 * Skills family page — docs/design/18 §3. Envelope master + binding explain.
 * Import/Bind/Revoke are class-A daemon calls. Content ≠ permission is a
 * standing caption, not a dismissible banner.
 */
export function SkillsPage() {
  const [selectedId, setSelectedId] = useState<string | undefined>();
  const [receipt, setReceipt] = useState<string | undefined>();

  const refreshList = useCallback(async () => {
    await fetchProjection(
      appProjections,
      resourceListKey("skill"),
      resourceListPath("skill"),
      "management",
      projectResourceList,
    );
  }, []);

  useEffect(() => {
    void refreshList();
  }, [refreshList]);

  const list = useProjection<ResourceListView>(resourceListKey("skill"));
  const rows = list.data?.resources ?? [];
  const selected = rows.find((row) => row.id === selectedId);
  const clearInspector = useCallback(() => setSelectedId(undefined), []);
  useInspectorClear(selectedId, clearInspector);

  useEffect(() => {
    if (selectedId && !rows.some((row) => row.id === selectedId) && list.status === "ready") {
      setSelectedId(undefined);
    }
  }, [list.status, rows, selectedId]);

  useEffect(() => {
    if (!selectedId) {
      return;
    }
    void fetchProjection(
      appProjections,
      skillExplainKey(selectedId),
      skillExplainPath(selectedId),
      "management",
      projectSkillExplain,
    );
  }, [selectedId]);

  const explain = useProjection<SkillExplainView>(
    selectedId ? skillExplainKey(selectedId) : "resources:skill:explain:none",
  );
  const atBound = list.data ? envelopeAtBound(list.data) : false;
  const inventorySource =
    list.status === "loading" && list.data === undefined ? "pending" : "answered";

  return (
    <section>
      <PageHeader
        title="Skills"
        lede="Bindings the Resource Manager will list. A binding is not a package and not a permission."
      />
      <p className="cp-quiet" data-annotation="skill-permission">
        {SKILL_PERMISSION_ANNOTATION}
      </p>
      <p className="cp-next">
        <Link to="/resources">Resources hub</Link>
        {" · "}
        <button type="button" className="cp-button" onClick={() => void refreshList()}>
          Refresh
        </button>{" "}
        <span className="cp-quiet">This space refreshes only when you ask.</span>
      </p>
      <HonestyNote>
        List is <code>GET /management/resource/v1/list?family=skill</code> (bindings, limit 64).
        Explain is <code>GET /management/resource/v1/skill/binding/explain</code>. Import, bind and
        revoke are class-A daemon calls on this page and are not drawn as hub controls.{" "}
        {SKILL_IMPORT_HONESTY}
      </HonestyNote>
      <ProjectionState projection={list} what="Skill binding list" />
      {receipt ? <ReceiptLine>{receipt}</ReceiptLine> : null}
      <div className="cp-mi">
        <div className="cp-master">
          {inventorySource === "pending" ? null : rows.length === 0 &&
            (list.status === "ready" || list.status === "empty") ? (
            <EmptyState title="No skill bindings">
              Bindings arrive through Import and Bind on this page. This list is not packages.
            </EmptyState>
          ) : rows.length > 0 ? (
            <MasterList
              caption="Skill bindings"
              columns={[
                {
                  key: "id",
                  header: "Binding",
                  mono: true,
                  render: (row: ResourceEnvelope) => row.id,
                },
                {
                  key: "health",
                  header: "Health",
                  render: (row: ResourceEnvelope) => row.health ?? "unknown",
                },
              ]}
              rows={rows}
              rowKey={(row) => row.id}
              selectedKey={selectedId}
              onSelect={(row) => setSelectedId(row.id)}
            />
          ) : null}
          {list.data ? (
            <p className="cp-quiet">{skillMasterFooter(rows.length, atBound)}</p>
          ) : null}
          <ImportForm
            onImported={async (packageId, revisionId) => {
              setReceipt(
                `Imported package ${packageId} revision ${revisionId}. Import is not a binding and grants no capability.`,
              );
              await refreshList();
            }}
          />
          <BindForm
            onBound={async (bindingId) => {
              setReceipt(`Bound ${bindingId}. Binding is not a capability grant.`);
              await refreshList();
              setSelectedId(bindingId);
            }}
          />
        </div>
        <SkillInspector
          selected={selected}
          explain={explain}
          onRevoked={async () => {
            setReceipt(`Revoked ${selectedId}. ${SKILL_REVOKE_CONSEQUENCE}`);
            await refreshList();
          }}
        />
      </div>
    </section>
  );
}

function SkillInspector({
  selected,
  explain,
  onRevoked,
}: {
  selected?: ResourceEnvelope;
  explain: Projection<SkillExplainView>;
  onRevoked: () => Promise<void>;
}) {
  if (!selected) {
    return (
      <Inspector title="Skill binding">
        <p className="cp-quiet">Select a binding to explain it. Nothing is inferred.</p>
      </Inspector>
    );
  }

  const missing =
    explain.status === "unknown" ||
    explain.status === "not-run" ||
    explain.status === "denied";

  return (
    <Inspector title={selected.id}>
      <ProjectionState projection={explain} what="Skill binding explain" />
      {missing ? (
        <p className="cp-quiet">
          This id is not shown as empty and is not shown as a package. Named gap:{" "}
          <code className="cp-mono">{explain.error?.code ?? explain.status}</code>.
        </p>
      ) : null}
      <FactGrid
        facts={[
          { label: "binding", value: selected.id },
          { label: "list health", value: selected.health ?? "unknown" },
          { label: "status", value: explain.data?.status ?? "unknown" },
          { label: "revision", value: explain.data?.revisionId ?? "unknown" },
          { label: "package", value: explain.data?.packageId ?? "unknown" },
          { label: "workspace", value: explain.data?.workspaceScope ?? "unknown" },
          { label: "target kind", value: explain.data?.targetKind ?? "unknown" },
          { label: "target", value: explain.data?.targetRef ?? "unknown" },
          { label: "manifest digest", value: explain.data?.manifestDigest ?? "unknown" },
          { label: "content digest", value: explain.data?.contentDigest ?? "unknown" },
          {
            label: "compatibility",
            value:
              explain.data?.compatibility ??
              "unknown (binding explain does not carry it)",
          },
          { label: "revocation", value: explain.data?.revocationReason ?? "none observed" },
        ]}
      />
      {explain.data && !missing ? (
        <RevokeForm bindingId={selected.id} onRevoked={onRevoked} />
      ) : null}
    </Inspector>
  );
}

function ImportForm({
  onImported,
}: {
  onImported: (packageId: string, revisionId: string) => Promise<void>;
}) {
  const [message, setMessage] = useState<string | undefined>();
  const [preview, setPreview] = useState<SkillImportPreview | undefined>();

  function prepare(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    const workspaceScope = String(data.get("workspace_scope") ?? "").trim();
    const localSourcePath = String(data.get("local_source_path") ?? "").trim();
    const provenanceRef = String(data.get("provenance_ref") ?? "").trim();
    const manifestDigest = String(data.get("manifest_digest") ?? "").trim();
    const contentDigest = String(data.get("content_digest") ?? "").trim();
    const compatibility = String(data.get("compatibility") ?? "").trim() || "compatible";
    if (!workspaceScope || !localSourcePath || !manifestDigest || !contentDigest) {
      setPreview(undefined);
      setMessage("Import requires workspace_scope, local_source_path, manifest_digest, and content_digest.");
      return;
    }
    setMessage(undefined);
    setPreview({
      packageId: uuidV7(),
      revisionId: uuidV7(),
      workspaceScope,
      localSourcePath,
      provenanceRef: provenanceRef || `file://${localSourcePath}`,
      manifestDigest,
      contentDigest,
      compatibility,
    });
  }

  async function submit() {
    if (!preview) {
      return;
    }
    const result = await readJson("/management/resource/v1/skill/import", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(skillImportBody(preview)),
    });
    const body = asRecord(result.body);
    if (result.status === 201 && body.status === "imported") {
      const packageId = String(body.package_id ?? preview.packageId);
      const revisionId = String(body.revision_id ?? preview.revisionId);
      setPreview(undefined);
      setMessage(undefined);
      await onImported(packageId, revisionId);
      return;
    }
    setMessage(
      `${asRecord(body.error).code ?? body.code ?? `HTTP_${result.status}`} — import did not persist a package.`,
    );
  }

  return (
    <section className="cp-section" aria-label="Import">
      <h3 className="cp-section-title">Import</h3>
      <p className="cp-quiet">{SKILL_IMPORT_HONESTY}</p>
      <form className="cp-form" data-skill-form="import" onSubmit={prepare}>
        <label className="cp-field">
          Workspace scope
          <input name="workspace_scope" required placeholder="workspace://personal/…" />
        </label>
        <label className="cp-field">
          Local source path
          <input name="local_source_path" required placeholder="skills/example" />
        </label>
        <label className="cp-field">
          Provenance ref
          <input name="provenance_ref" placeholder="file://workspace/skills/example" />
        </label>
        <label className="cp-field">
          Manifest digest
          <input name="manifest_digest" required className="cp-mono" placeholder="sha256:…" />
        </label>
        <label className="cp-field">
          Content digest
          <input name="content_digest" required className="cp-mono" placeholder="sha256:…" />
        </label>
        <label className="cp-field">
          Compatibility
          <input name="compatibility" defaultValue="compatible" />
        </label>
        <button type="submit" className="cp-button">
          Preview import
        </button>
      </form>
      {message ? <p className="cp-reason">{message}</p> : null}
      {preview ? (
        <ConfirmSurface
          title="Confirm import"
          consequences={`${SKILL_IMPORT_HONESTY} This calls POST /management/resource/v1/skill/import.`}
          targets={[
            `package_id ${preview.packageId}`,
            `revision_id ${preview.revisionId}`,
            `path ${preview.localSourcePath}`,
            `manifest ${preview.manifestDigest}`,
            `content ${preview.contentDigest}`,
            `compatibility ${preview.compatibility}`,
          ]}
          confirmLabel={`I am importing this path under ${preview.workspaceScope}`}
          actionLabel="Import"
          onConfirm={() => void submit()}
        />
      ) : null}
    </section>
  );
}

function BindForm({ onBound }: { onBound: (bindingId: string) => Promise<void> }) {
  const [message, setMessage] = useState<string | undefined>();
  const [preview, setPreview] = useState<SkillBindPreview | undefined>();

  function prepare(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    const revisionId = String(data.get("revision_id") ?? "").trim();
    const workspaceScope = String(data.get("workspace_scope") ?? "").trim();
    const targetKind = String(data.get("target_kind") ?? "").trim();
    const targetRef = String(data.get("target_ref") ?? "").trim();
    if (!revisionId || !workspaceScope || !targetKind || !targetRef) {
      setPreview(undefined);
      setMessage("Bind requires revision_id, workspace_scope, target_kind, and target_ref.");
      return;
    }
    setMessage(undefined);
    setPreview({
      bindingId: uuidV7(),
      revisionId,
      workspaceScope,
      targetKind,
      targetRef,
    });
  }

  async function submit() {
    if (!preview) {
      return;
    }
    const result = await readJson("/management/resource/v1/skill/bind", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(skillBindBody(preview)),
    });
    const body = asRecord(result.body);
    if (result.status === 201 && body.status === "bound") {
      const bindingId = String(body.binding_id ?? preview.bindingId);
      setPreview(undefined);
      setMessage(undefined);
      await onBound(bindingId);
      return;
    }
    setMessage(
      `${asRecord(body.error).code ?? body.code ?? `HTTP_${result.status}`} — bind did not persist a binding.`,
    );
  }

  return (
    <section className="cp-section" aria-label="Bind">
      <h3 className="cp-section-title">Bind</h3>
      <p className="cp-quiet">
        Bind is CAS on binding_id + revision_id. Binding is not a capability grant.
      </p>
      <form className="cp-form" data-skill-form="bind" onSubmit={prepare}>
        <label className="cp-field">
          Revision id
          <input name="revision_id" required className="cp-mono" />
        </label>
        <label className="cp-field">
          Workspace scope
          <input name="workspace_scope" required />
        </label>
        <label className="cp-field">
          Target kind
          <input name="target_kind" required placeholder="workspace" />
        </label>
        <label className="cp-field">
          Target ref
          <input name="target_ref" required />
        </label>
        <button type="submit" className="cp-button">
          Preview bind
        </button>
      </form>
      {message ? <p className="cp-reason">{message}</p> : null}
      {preview ? (
        <ConfirmSurface
          title="Confirm bind"
          consequences="This calls POST /management/resource/v1/skill/bind. Binding is not a capability grant."
          targets={[
            `binding_id ${preview.bindingId}`,
            `revision_id ${preview.revisionId}`,
            `scope ${preview.workspaceScope}`,
            `target ${preview.targetKind}:${preview.targetRef}`,
          ]}
          confirmLabel={`I am binding revision ${preview.revisionId} under ${preview.workspaceScope}`}
          actionLabel="Bind"
          onConfirm={() => void submit()}
        />
      ) : null}
    </section>
  );
}

function RevokeForm({
  bindingId,
  onRevoked,
}: {
  bindingId: string;
  onRevoked: () => Promise<void>;
}) {
  const [message, setMessage] = useState<string | undefined>();
  const [preview, setPreview] = useState<{ revocationId: string; reason: string } | undefined>();

  function prepare(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const reason = String(new FormData(event.currentTarget).get("reason") ?? "").trim();
    if (!reason) {
      setPreview(undefined);
      setMessage("Revoke requires a reason.");
      return;
    }
    setMessage(undefined);
    setPreview({ revocationId: uuidV7(), reason });
  }

  async function submit() {
    if (!preview) {
      return;
    }
    const result = await readJson("/management/resource/v1/skill/binding/revoke", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        binding_id: bindingId,
        revocation_id: preview.revocationId,
        reason: preview.reason,
      }),
    });
    const body = asRecord(result.body);
    if (result.status === 201 && body.status === "revoked") {
      setPreview(undefined);
      setMessage(`Revoked ${bindingId}. ${SKILL_REVOKE_CONSEQUENCE}`);
      await onRevoked();
      return;
    }
    setMessage(
      `${asRecord(body.error).code ?? body.code ?? `HTTP_${result.status}`} — revoke did not persist a revocation.`,
    );
  }

  return (
    <section className="cp-subblock" aria-label="Revoke">
      <h4 className="cp-section-title">Revoke</h4>
      <p className="cp-quiet">{SKILL_REVOKE_CONSEQUENCE}</p>
      <form className="cp-form" data-skill-form="revoke" onSubmit={prepare}>
        <label className="cp-field">
          Reason
          <input name="reason" required />
        </label>
        <button type="submit" className="cp-button">
          Preview revoke
        </button>
      </form>
      {message ? <p className="cp-reason">{message}</p> : null}
      {preview ? (
        <ConfirmSurface
          title="Confirm revoke"
          consequences={SKILL_REVOKE_CONSEQUENCE}
          targets={[
            `binding_id ${bindingId}`,
            `revocation_id ${preview.revocationId}`,
            `reason ${preview.reason}`,
          ]}
          confirmLabel={`I am revoking ${bindingId}`}
          actionLabel="Revoke"
          danger
          onConfirm={() => void submit()}
        />
      ) : null}
    </section>
  );
}

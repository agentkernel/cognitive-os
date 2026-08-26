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
  MEMORY_FORGET_CONSEQUENCE,
  MEMORY_RETENTION_CAP_DAYS,
  MEMORY_SEARCH_UNAVAILABLE,
  memoryInspectKey,
  memoryInspectPath,
  memoryMasterFooter,
  memoryObjectKey,
  memoryObjectPath,
  projectMemoryExplain,
  projectMemoryInspect,
  retentionExpiryUnix,
  type MemoryExplainView,
} from "../../data/projections/memory";
import {
  envelopeAtBound,
  projectResourceList,
  resourceListKey,
  resourceListPath,
  type ResourceEnvelope,
  type ResourceListView,
} from "../../data/projections/resources";
import { appProjections, type Projection } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { useInspectorClear } from "../../shell/useInspectorClear";
import { HonestyNote } from "../../state/HonestyNote";
import { ProjectionState } from "../providers/ProjectionState";

/**
 * Memory family page — docs/design/18 §2. Envelope master + explain inspector.
 * Remember/Forget are class-A daemon calls. Content search is BD-6.
 */
export function MemoryPage() {
  const [selectedId, setSelectedId] = useState<string | undefined>();
  const [receipt, setReceipt] = useState<string | undefined>();

  const refreshList = useCallback(async () => {
    await fetchProjection(
      appProjections,
      resourceListKey("memory"),
      resourceListPath("memory"),
      "management",
      projectResourceList,
    );
  }, []);

  useEffect(() => {
    void refreshList();
  }, [refreshList]);

  const list = useProjection<ResourceListView>(resourceListKey("memory"));
  const rows = list.data?.resources ?? [];
  const selected = rows.find((row) => row.id === selectedId);
  const clearInspector = useCallback(() => setSelectedId(undefined), []);
  useInspectorClear(selectedId, clearInspector);

  useEffect(() => {
    if (selectedId && !rows.some((row) => row.id === selectedId) && list.status === "ready") {
      setSelectedId(undefined);
    }
  }, [list.status, rows, selectedId]);

  const loadExplain = useCallback(async (id: string) => {
    await fetchProjection(
      appProjections,
      memoryInspectKey(id),
      memoryInspectPath(id),
      "management",
      projectMemoryInspect,
    );
    await fetchProjection(
      appProjections,
      memoryObjectKey(id),
      memoryObjectPath(id),
      "management",
      projectMemoryExplain,
    );
  }, []);

  useEffect(() => {
    if (selectedId) {
      void loadExplain(selectedId);
    }
  }, [loadExplain, selectedId]);

  const inspect = useProjection<ResourceEnvelope>(
    selectedId ? memoryInspectKey(selectedId) : "resources:memory:inspect:none",
  );
  const explain = useProjection<MemoryExplainView>(
    selectedId ? memoryObjectKey(selectedId) : "resources:memory:object:none",
  );

  const atBound = list.data ? envelopeAtBound(list.data) : false;
  const inventorySource =
    list.status === "loading" && list.data === undefined ? "pending" : "answered";

  return (
    <section>
      <PageHeader
        title="Memory"
        lede="Admitted objects the Resource Manager will list. Content search is not on this page."
      />
      <p className="cp-next">
        <Link to="/resources">Resources hub</Link>
        {" · "}
        <button type="button" className="cp-button" onClick={() => void refreshList()}>
          Refresh
        </button>{" "}
        <span className="cp-quiet">This space refreshes only when you ask.</span>
      </p>

      <HonestyNote>
        List is <code>GET /management/resource/v1/list?family=memory</code> (non-tombstoned, limit
        64). Explain is <code>GET /management/resource/v1/memory/object</code>. {MEMORY_SEARCH_UNAVAILABLE}{" "}
        Retention is capped at {MEMORY_RETENTION_CAP_DAYS} days.
      </HonestyNote>

      <ProjectionState projection={list} what="Memory list" />
      {receipt ? <ReceiptLine>{receipt}</ReceiptLine> : null}

      <div className="cp-mi">
        <div className="cp-master">
          {inventorySource === "pending" ? null : rows.length === 0 &&
            (list.status === "ready" || list.status === "empty") ? (
            <EmptyState title="No admitted Memory objects">
              Objects arrive through Remember on this page. Tombstones are not in this list.
            </EmptyState>
          ) : rows.length > 0 ? (
            <MasterList
              caption="Admitted Memory objects"
              columns={[
                {
                  key: "id",
                  header: "Id",
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
            <p className="cp-quiet">{memoryMasterFooter(rows.length, atBound)}</p>
          ) : null}
          <RememberForm
            onRemembered={async (id) => {
              setReceipt(`Remembered ${id}. Admission is not a search index and is not Task completion.`);
              await refreshList();
              setSelectedId(id);
            }}
          />
        </div>
        <MemoryInspector
          selected={selected}
          inspect={inspect}
          explain={explain}
          onForgotten={async () => {
            setReceipt(`Forgotten ${selectedId}. ${MEMORY_FORGET_CONSEQUENCE}`);
            await refreshList();
          }}
        />
      </div>
    </section>
  );
}

function MemoryInspector({
  selected,
  inspect,
  explain,
  onForgotten,
}: {
  selected?: ResourceEnvelope;
  inspect: Projection<ResourceEnvelope>;
  explain: Projection<MemoryExplainView>;
  onForgotten: () => Promise<void>;
}) {
  if (!selected) {
    return (
      <Inspector title="Memory">
        <p className="cp-quiet">Select an admitted object to explain it. Nothing is inferred.</p>
      </Inspector>
    );
  }

  const missing =
    inspect.status === "unknown" ||
    inspect.status === "not-run" ||
    inspect.status === "denied" ||
    explain.status === "unknown" ||
    explain.status === "not-run" ||
    explain.status === "denied";

  return (
    <Inspector title={selected.id}>
      <ProjectionState projection={inspect} what="Memory inspect" />
      <ProjectionState projection={explain} what="Memory explain" />
      {missing ? (
        <p className="cp-quiet">
          This id is not shown as empty and is not shown as current content. Named gap:{" "}
          <code className="cp-mono">
            {inspect.error?.code ?? explain.error?.code ?? inspect.status}
          </code>
          .
        </p>
      ) : null}
      <FactGrid
        facts={[
          { label: "id", value: selected.id },
          { label: "health", value: inspect.data?.health ?? selected.health ?? "unknown" },
          { label: "candidate", value: explain.data?.candidateId ?? "unknown" },
          { label: "decision", value: explain.data?.decisionId ?? "unknown" },
        ]}
      />
      <h4 className="cp-section-title">Canonical content</h4>
      {explain.data?.canonicalJson ? (
        <pre className="cp-mono">{explain.data.canonicalJson}</pre>
      ) : (
        <p className="cp-quiet">canonical content unknown (the explain envelope did not carry it)</p>
      )}
      {explain.data ? <ForgetForm memoryId={selected.id} onForgotten={onForgotten} /> : null}
    </Inspector>
  );
}

function RememberForm({ onRemembered }: { onRemembered: (id: string) => Promise<void> }) {
  const [message, setMessage] = useState<string | undefined>();
  const [preview, setPreview] = useState<
    { text: string; scope: string; purpose: string; days: number; unix: number } | undefined
  >();

  function prepare(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    const text = String(data.get("text") ?? "").trim();
    const scope = String(data.get("governance_scope") ?? "").trim();
    const purpose = String(data.get("purpose") ?? "").trim() || "task_execution";
    const days = Number(data.get("retention_days"));
    const retention = retentionExpiryUnix(days, Math.floor(Date.now() / 1000));
    if (!text || !scope) {
      setPreview(undefined);
      setMessage("Remember requires text and governance_scope.");
      return;
    }
    if (!retention.ok) {
      setPreview(undefined);
      setMessage(retention.reason);
      return;
    }
    setMessage(undefined);
    setPreview({ text, scope, purpose, days, unix: retention.unix });
  }

  async function submit() {
    if (!preview) {
      return;
    }
    const result = await readJson("/management/resource/v1/memory/remember", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        text: preview.text,
        governance_scope: preview.scope,
        target_scope: preview.scope,
        purpose: preview.purpose,
        retention_expires_at_unix_seconds: preview.unix,
      }),
    });
    const body = asRecord(result.body);
    if (result.status === 201 && typeof body.memory_id === "string") {
      setPreview(undefined);
      setMessage(undefined);
      await onRemembered(body.memory_id);
      return;
    }
    setMessage(
      `${asRecord(body.error).code ?? body.code ?? `HTTP_${result.status}`} — remember did not admit an object.`,
    );
  }

  return (
    <section className="cp-section" aria-label="Remember">
      <h3 className="cp-section-title">Remember</h3>
      <p className="cp-quiet">
        Unsealed public remember. The daemon seals the candidate. {MEMORY_SEARCH_UNAVAILABLE} Retention
        cap {MEMORY_RETENTION_CAP_DAYS} days.
      </p>
      <form className="cp-form" onSubmit={prepare}>
        <label className="cp-field">
          Text
          <textarea name="text" rows={3} required />
        </label>
        <label className="cp-field">
          Governance scope
          <input name="governance_scope" required placeholder="workspace://personal/…" />
        </label>
        <label className="cp-field">
          Purpose
          <input name="purpose" defaultValue="task_execution" />
        </label>
        <label className="cp-field">
          Retention days (≤ {MEMORY_RETENTION_CAP_DAYS})
          <input name="retention_days" type="number" min={1} max={MEMORY_RETENTION_CAP_DAYS} defaultValue={90} />
        </label>
        <button type="submit" className="cp-button">
          Preview remember
        </button>
      </form>
      {message ? <p className="cp-reason">{message}</p> : null}
      {preview ? (
        <ConfirmSurface
          title="Confirm remember"
          consequences="This calls POST /management/resource/v1/memory/remember. Admission is not execution and not a search index."
          targets={[
            `scope ${preview.scope}`,
            `purpose ${preview.purpose}`,
            `retention_expires_at_unix_seconds ${preview.unix}`,
          ]}
          confirmLabel={`I am admitting this text into Memory under ${preview.scope}`}
          actionLabel="Remember"
          onConfirm={() => void submit()}
        />
      ) : null}
    </section>
  );
}

function ForgetForm({
  memoryId,
  onForgotten,
}: {
  memoryId: string;
  onForgotten: () => Promise<void>;
}) {
  const [reason, setReason] = useState("");
  const [message, setMessage] = useState<string | undefined>();
  const [armed, setArmed] = useState(false);

  async function submit() {
    const result = await readJson("/management/resource/v1/memory/forget", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ memory_id: memoryId, reason }),
    });
    const body = asRecord(result.body);
    if (result.status === 201 && body.status === "forgotten") {
      setArmed(false);
      setMessage(`Forgotten ${memoryId}. ${MEMORY_FORGET_CONSEQUENCE}`);
      await onForgotten();
      return;
    }
    setMessage(
      `${asRecord(body.error).code ?? body.code ?? `HTTP_${result.status}`} — forget did not tombstone this object.`,
    );
  }

  return (
    <section className="cp-subblock" aria-label="Forget">
      <h4 className="cp-section-title">Forget</h4>
      <p className="cp-quiet">{MEMORY_FORGET_CONSEQUENCE}</p>
      <label className="cp-field">
        Reason
        <input value={reason} onChange={(event) => setReason(event.target.value)} />
      </label>
      <button
        type="button"
        className="cp-button"
        disabled={!reason.trim()}
        onClick={() => setArmed(true)}
      >
        Preview forget
      </button>
      {message ? <p className="cp-reason">{message}</p> : null}
      {armed ? (
        <ConfirmSurface
          title="Confirm forget"
          consequences={MEMORY_FORGET_CONSEQUENCE}
          targets={[`memory_id ${memoryId}`, `reason ${reason}`]}
          confirmLabel={`I am tombstoning ${memoryId}`}
          actionLabel="Forget"
          danger
          onConfirm={() => void submit()}
        />
      ) : null}
    </section>
  );
}

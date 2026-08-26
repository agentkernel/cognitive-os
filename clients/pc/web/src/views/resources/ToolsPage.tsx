import { useCallback, useEffect, useState } from "react";
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
  TOOL_CATALOG_KEY,
  TOOL_CATALOG_PATH,
  TOOL_QUARANTINE_CONSEQUENCE,
  TOOL_READINESS_CAVEAT,
  TOOL_REVOKE_CONSEQUENCE,
  allowedToolMutations,
  projectToolCatalog,
  readinessLabel,
  toolMutationBody,
  toolMutationConsequence,
  toolMutationPath,
  type ToolCatalogRow,
  type ToolCatalogView,
  type ToolMutationKind,
} from "../../data/projections/tools";
import { appProjections, type Projection } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { useInspectorClear } from "../../shell/useInspectorClear";
import { HonestyNote } from "../../state/HonestyNote";
import { ProjectionState } from "../providers/ProjectionState";

/**
 * Tools family page — docs/design/18 §4. Catalog table + overlay mutations.
 * Enable/Disable/Quarantine/Revoke are class-A. Hub counts stay on the list
 * envelope; this table is the native catalog.
 */
export function ToolsPage() {
  const [selectedId, setSelectedId] = useState<string | undefined>();
  const [receipt, setReceipt] = useState<string | undefined>();

  const refresh = useCallback(async () => {
    await fetchProjection(
      appProjections,
      TOOL_CATALOG_KEY,
      TOOL_CATALOG_PATH,
      "management",
      projectToolCatalog,
    );
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const catalog = useProjection<ToolCatalogView>(TOOL_CATALOG_KEY);
  const rows = catalog.data?.resources ?? [];
  const selected = rows.find((row) => row.operationId === selectedId);
  const clearInspector = useCallback(() => setSelectedId(undefined), []);
  useInspectorClear(selectedId, clearInspector);

  useEffect(() => {
    if (selectedId && !rows.some((row) => row.operationId === selectedId) && catalog.status === "ready") {
      setSelectedId(undefined);
    }
  }, [catalog.status, rows, selectedId]);

  const inventorySource =
    catalog.status === "loading" && catalog.data === undefined ? "pending" : "answered";

  return (
    <section>
      <PageHeader
        title="Tools"
        lede="Native catalog operations and their overlay lifecycle. This is not a card wall."
      />
      <p className="cp-quiet" data-annotation="tool-readiness">
        {TOOL_READINESS_CAVEAT}
      </p>
      <p className="cp-next">
        <Link to="/resources">Resources hub</Link>
        {" · "}
        <button type="button" className="cp-button" onClick={() => void refresh()}>
          Refresh
        </button>{" "}
        <span className="cp-quiet">This space refreshes only when you ask.</span>
      </p>
      <HonestyNote>
        This table is <code>GET /management/resource/v1/tool</code> (catalog + overlay). Hub counts
        come from <code>list?family=tool</code> and are a different read. Enable, disable,
        quarantine and revoke POST only <code>operation_id</code>. {TOOL_QUARANTINE_CONSEQUENCE}{" "}
        {TOOL_REVOKE_CONSEQUENCE}
      </HonestyNote>
      <ProjectionState projection={catalog} what="Tool catalog" />
      {receipt ? <ReceiptLine>{receipt}</ReceiptLine> : null}
      <div className="cp-mi">
        <div className="cp-master">
          {inventorySource === "pending" ? null : rows.length === 0 &&
            (catalog.status === "ready" || catalog.status === "empty") ? (
            <EmptyState title="No registered tools">
              The native catalog did not list operations. This is not an invented empty family.
            </EmptyState>
          ) : rows.length > 0 ? (
            <MasterList
              caption="Registered tools"
              columns={[
                {
                  key: "operation",
                  header: "Operation",
                  mono: true,
                  render: (row: ToolCatalogRow) => row.operationId,
                },
                {
                  key: "risk",
                  header: "Risk",
                  render: (row: ToolCatalogRow) => row.risk ?? "unknown",
                },
                {
                  key: "lifecycle",
                  header: "Lifecycle",
                  render: (row: ToolCatalogRow) => row.lifecycle,
                },
                {
                  key: "readiness",
                  header: "Readiness",
                  render: (row: ToolCatalogRow) => readinessLabel(row.executionReadiness),
                },
              ]}
              rows={rows}
              rowKey={(row) => row.operationId}
              selectedKey={selectedId}
              onSelect={(row) => setSelectedId(row.operationId)}
            />
          ) : null}
          <p className="cp-quiet">* {TOOL_READINESS_CAVEAT}</p>
        </div>
        <ToolInspector
          selected={selected}
          catalog={catalog}
          onMutated={async (kind, operationId) => {
            setReceipt(`${kind} ${operationId}. Overlay write only. ${TOOL_READINESS_CAVEAT}`);
            await refresh();
            setSelectedId(operationId);
          }}
        />
      </div>
    </section>
  );
}

function ToolInspector({
  selected,
  catalog,
  onMutated,
}: {
  selected?: ToolCatalogRow;
  catalog: Projection<ToolCatalogView>;
  onMutated: (kind: ToolMutationKind, operationId: string) => Promise<void>;
}) {
  if (!selected) {
    return (
      <Inspector title="Tool">
        <p className="cp-quiet">Select an operation to inspect it. Nothing is inferred.</p>
      </Inspector>
    );
  }
  return (
    <Inspector title={selected.operationId}>
      <ProjectionState projection={catalog} what="Tool catalog" />
      <FactGrid
        facts={[
          { label: "operation", value: selected.operationId },
          { label: "action", value: selected.action ?? "unknown" },
          { label: "family", value: selected.family ?? "unknown" },
          { label: "risk", value: selected.risk ?? "unknown" },
          { label: "lifecycle", value: selected.lifecycle },
          { label: "readiness", value: readinessLabel(selected.executionReadiness) },
          { label: "descriptor digest", value: selected.descriptorDigest ?? "unknown" },
          {
            label: "agent exposed",
            value: selected.agentExposed ? "true (enabled and execution-ready)" : "false",
          },
        ]}
      />
      <p className="cp-quiet">{TOOL_READINESS_CAVEAT}</p>
      {allowedToolMutations(selected.lifecycle).length === 0 ? (
        <p className="cp-quiet">
          {selected.lifecycle === "revoked"
            ? TOOL_REVOKE_CONSEQUENCE
            : "No overlay mutation is offered for this lifecycle."}
        </p>
      ) : (
        allowedToolMutations(selected.lifecycle).map((kind) => (
          <ToolMutationForm
            key={`${selected.operationId}:${kind}`}
            kind={kind}
            row={selected}
            onMutated={onMutated}
          />
        ))
      )}
    </Inspector>
  );
}

function ToolMutationForm({
  kind,
  row,
  onMutated,
}: {
  kind: ToolMutationKind;
  row: ToolCatalogRow;
  onMutated: (kind: ToolMutationKind, operationId: string) => Promise<void>;
}) {
  const [armed, setArmed] = useState(false);
  const [message, setMessage] = useState<string | undefined>();
  const label = kind.charAt(0).toUpperCase() + kind.slice(1);
  const danger = kind === "quarantine" || kind === "revoke";

  async function submit() {
    const result = await readJson(toolMutationPath(kind), "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(toolMutationBody(row.operationId)),
    });
    const body = asRecord(result.body);
    if (result.status === 200 && body.kind === "tool.lifecycle.mutation") {
      setArmed(false);
      setMessage(undefined);
      await onMutated(kind, row.operationId);
      return;
    }
    setMessage(
      `${asRecord(body.error).code ?? body.code ?? `HTTP_${result.status}`} — ${kind} did not write the overlay.`,
    );
  }

  return (
    <section className="cp-subblock" aria-label={label}>
      <h4 className="cp-section-title">{label}</h4>
      <p className="cp-quiet">{toolMutationConsequence(kind)}</p>
      <button type="button" className="cp-button" onClick={() => setArmed(true)}>
        Preview {kind}
      </button>
      {message ? <p className="cp-reason">{message}</p> : null}
      {armed ? (
        <ConfirmSurface
          title={`Confirm ${kind}`}
          consequences={toolMutationConsequence(kind)}
          targets={[`operation_id ${row.operationId}`, `current lifecycle ${row.lifecycle}`]}
          confirmLabel={`I am writing ${kind} for ${row.operationId}`}
          actionLabel={label}
          danger={danger}
          onConfirm={() => void submit()}
        />
      ) : null}
    </section>
  );
}

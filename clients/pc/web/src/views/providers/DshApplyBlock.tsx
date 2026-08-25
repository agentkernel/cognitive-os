import { useEffect, useState } from "react";
import { readJson } from "../../api";
import { fetchProjection } from "../../data/fetchProjection";
import { asRecord } from "../../data/projections";
import {
  DSH_RUNTIME_KEY,
  DSH_SELECTED_KEY,
  projectDshRuntime,
  projectDshSelected,
  type BindingView,
  type DshRuntimeView,
  type DshSelectedView,
  type ProviderModel,
} from "../../data/projections/providers";
import { appProjections } from "../../data/store";
import { useProjection } from "../../data/useProjection";
import { acceptDshApply, bindingRevisionForCas } from "../../policy";

/**
 * DshApplyBlock — the fail-closed "apply to running dsh" (docs/design/17
 * footer). The gate (policy.acceptDshApply, reused verbatim) requires an
 * active dsh binding on this account, the bound model in this account's
 * catalog, and an ACTIVE runtime with a live process; gate reasons are
 * shown. A stored binding is never equated with a runtime apply; the
 * expected revision comes from the active binding only.
 */
export function DshApplyBlock({
  accountId,
  bindings,
  models,
  onChanged,
}: {
  accountId: string;
  bindings: BindingView[];
  models: ProviderModel[];
  onChanged: () => void;
}) {
  const runtime = useProjection<DshRuntimeView>(DSH_RUNTIME_KEY);
  const selected = useProjection<DshSelectedView>(DSH_SELECTED_KEY);
  const [message, setMessage] = useState("");
  const [applying, setApplying] = useState(false);

  useEffect(() => {
    void fetchProjection(
      appProjections,
      DSH_RUNTIME_KEY,
      "/personal/dsh/runtime",
      "management",
      projectDshRuntime,
    );
    void fetchProjection(
      appProjections,
      DSH_SELECTED_KEY,
      "/provider/v1/dsh/selected-model",
      "management",
      projectDshSelected,
    );
  }, []);

  const dshRow = bindings.find(
    (row) => row.status === "active" && (row.agent === "dsh" || row.agent.endsWith("dsh")),
  );
  const onThisAccount = dshRow?.accountId === accountId;
  const catalogIds = models.map((model) => model.id);
  const gate = acceptDshApply({
    agent: "dsh",
    bindingStatus: onThisAccount ? dshRow?.status : undefined,
    modelId: onThisAccount ? dshRow?.modelId : undefined,
    catalogModelIds: onThisAccount && catalogIds.length > 0 ? catalogIds : undefined,
    runtimeState: runtime.data?.state,
    processAlive: runtime.data?.processAlive,
  });

  async function apply() {
    setApplying(true);
    setMessage("");
    const result = await readJson("/personal/dsh/runtime", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        schema_version: 1,
        surface: "personal-dsh-runtime",
        op: "apply",
        expected_revision: bindingRevisionForCas(dshRow),
      }),
    });
    setApplying(false);
    const applied = asRecord(result.body);
    const appliedModel = String(applied.applied_model ?? dshRow?.modelId ?? "unknown");
    setMessage(
      result.ok
        ? applied.restart_performed === true
          ? `Applied ${appliedModel}. Native Models now lists this bound account catalog.`
          : `Applied ${appliedModel}. Overlay written; refresh dsh after Cos web reloads.`
        : `HTTP ${result.status} ${String(applied.code ?? "")}`,
    );
    onChanged();
  }

  return (
    <div className="cp-subblock">
      <h4 className="cp-section-title">Apply Cos model to running dsh</h4>
      <p className="cp-quiet">
        A stored binding is not a runtime apply. Apply publishes the Cos dsh binding so the
        native dsh panel shows{" "}
        <code className="cp-mono">{selected.data?.selectedModel ?? "unset"}</code>. Runtime{" "}
        <code className="cp-mono">{runtime.data?.state ?? "unknown"}</code>. The dsh snapshot is
        an observation (candidate_only), not an authority read.
      </p>
      <button
        type="button"
        className="cp-button"
        disabled={applying || !gate.ok}
        onClick={() => void apply()}
      >
        Apply to running dsh
      </button>
      {!gate.ok ? (
        <p className="cp-quiet" role="note">
          {gate.reason}
        </p>
      ) : null}
      {message ? <p role="status">{message}</p> : null}
    </div>
  );
}

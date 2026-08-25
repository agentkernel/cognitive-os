import { useState } from "react";
import { readJson } from "../../api";
import { ConfirmSurface } from "../../components/ConfirmSurface";
import { ReceiptLine } from "../../components/ReceiptLine";
import { fetchProjection } from "../../data/fetchProjection";
import { asRecord } from "../../data/projections";
import {
  projectBindings,
  type BindingView,
  type ProviderModel,
} from "../../data/projections/providers";
import { appProjections, type Projection } from "../../data/store";
import {
  acceptBindingMutation,
  bindingRevisionForCas,
  dispatchAllowed,
} from "../../policy";
import { StateChip } from "../../state/StateChip";
import { readBindingState } from "../../state/stateMap";
import { DshApplyBlock } from "./DshApplyBlock";
import { ProjectionState } from "./ProjectionState";

export const BINDINGS_KEY = "bindings:all";

/**
 * Bindings section — docs/design/17 §2.3. One active account + model per
 * agent, set with a revision-aware preview naming the exact tuple and the
 * expected CAS revision. 409 PROVIDER_BINDING_REVISION_STALE re-reads the
 * authority state and presents a fresh preview for re-confirmation — never
 * a silent retry. Revoked rows never supply the CAS revision. Fallback and
 * per-request override are policy text, not offered options.
 */
export function BindingsSection({
  accountId,
  accountStatus,
  bindings,
  models,
  onChanged,
}: {
  accountId: string;
  accountStatus: string;
  bindings: Projection<BindingView[]>;
  models: ProviderModel[];
  onChanged: () => void;
}) {
  const [agent, setAgent] = useState("pi");
  const [modelId, setModelId] = useState("");
  const [stale, setStale] = useState(false);
  const [message, setMessage] = useState(
    "At most one active account + model per agent.",
  );
  const [receipt, setReceipt] = useState<string | undefined>();
  const [pendingRemove, setPendingRemove] = useState<string | undefined>();

  const rows = bindings.data ?? [];
  const accountRows = rows.filter((row) => row.accountId === accountId);
  const current = rows.find(
    (row) => row.status === "active" && (row.agent === agent || row.agent.endsWith(agent)),
  );
  // Only an active binding occupies expected_revision; revoked/missing → 0.
  const expected = bindingRevisionForCas(current);

  async function setBinding() {
    setStale(false);
    setReceipt(undefined);
    const gate = acceptBindingMutation({
      expectedRevision: expected,
      currentRevision: bindingRevisionForCas(current),
    });
    if (!gate.ok) {
      setMessage(gate.reason);
      return;
    }
    const result = await readJson("/management/agent-bindings", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        agent,
        account_id: accountId,
        model_id: modelId,
        expected_revision: expected,
      }),
    });
    const code = String(asRecord(result.body).code ?? "");
    if (result.ok) {
      const callable = dispatchAllowed({ accountStatus, bindingStatus: "active" });
      setReceipt(
        `Binding stored: agent ${agent} → account ${accountId} → model ${modelId} · revision ${expected}. Dispatch ${callable ? "allowed" : "blocked until the account is usable"}.`,
      );
      setMessage("At most one active account + model per agent.");
      onChanged();
      return;
    }
    if (result.status === 409 && code === "PROVIDER_BINDING_REVISION_STALE") {
      setStale(true);
      setMessage(
        "The binding changed under you. The authority state was re-read — review the fresh preview and confirm again. No automatic retry was made.",
      );
      await fetchProjection(
        appProjections,
        BINDINGS_KEY,
        "/management/agent-bindings",
        "management",
        projectBindings,
      );
      return;
    }
    setMessage(`HTTP ${result.status} ${code}`);
  }

  async function removeBinding(target: string) {
    setReceipt(undefined);
    const result = await readJson("/management/agent-bindings/remove", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ agent: target }),
    });
    const code = String(asRecord(result.body).code ?? "");
    if (result.ok) {
      setReceipt(
        target === "dsh" || target.endsWith("/dsh")
          ? "dsh binding removed. Native Models drops that account catalog. Refresh dsh if the chrome is catching up."
          : `Binding for ${target} removed. The next set uses expected revision 0.`,
      );
      setMessage("At most one active account + model per agent.");
    } else {
      setMessage(
        result.status === 404 || code === "PROVIDER_CONTROL_NOT_FOUND"
          ? "No active binding to remove. Set a new model with expected revision 0."
          : `HTTP ${result.status} ${code}`,
      );
    }
    setPendingRemove(undefined);
    onChanged();
  }

  return (
    <>
      <ProjectionState projection={bindings} what="Agent bindings" />
      {bindings.status === "ready" ||
      bindings.status === "empty" ||
      bindings.status === "stale" ? (
        <>
          <p className="cp-quiet">
            One active <code className="cp-mono">account + provider + model</code> per agent.
            Fallback and per-request override are forbidden by policy and are not offered here.
            Unbound, revoked, or degraded accounts cannot dispatch.
          </p>
          {accountRows.length === 0 ? (
            <p className="cp-quiet">
              No bindings reference this account. Set one below — the first set uses expected
              revision 0.
            </p>
          ) : (
            <table className="cp-table">
              <caption>
                Bindings on this account — revoked rows never supply the CAS revision
              </caption>
              <thead>
                <tr>
                  <th scope="col">Agent</th>
                  <th scope="col">Model</th>
                  <th scope="col">Revision</th>
                  <th scope="col">Status</th>
                  <th scope="col" aria-label="actions" />
                </tr>
              </thead>
              <tbody>
                {accountRows.map((row) => {
                  const callable = dispatchAllowed({
                    accountStatus,
                    bindingStatus: row.status,
                  });
                  return (
                    <tr key={row.agent}>
                      <td className="cp-mono">{row.agent}</td>
                      <td className="cp-mono">{row.modelId}</td>
                      <td className="cp-mono">{row.revision ?? "unknown"}</td>
                      <td>
                        <StateChip reading={readBindingState(row.status, callable)} />
                      </td>
                      <td>
                        {row.status === "active" ? (
                          <button
                            type="button"
                            className="cp-button"
                            onClick={() => setPendingRemove(row.agent)}
                          >
                            Remove
                          </button>
                        ) : null}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          )}
          {pendingRemove ? (
            <ConfirmSurface
              title={`Remove binding for ${pendingRemove}`}
              consequences="The agent becomes non-callable for new dispatch; for dsh, the native overlay drops this account's models. Removing is separate from repairing the account."
              targets={[`agent: ${pendingRemove}`, `account: ${accountId}`]}
              confirmLabel={`Confirm removing the ${pendingRemove} binding`}
              actionLabel="Remove binding"
              danger
              onConfirm={() => void removeBinding(pendingRemove)}
            />
          ) : null}
          <div className="cp-panel">
            <h4 className="cp-section-title">Set or change binding</h4>
            <label className="cp-field">
              <span>Agent</span>
              <select
                name="binding_agent"
                value={agent}
                onChange={(event) => {
                  setAgent(event.target.value);
                  setStale(false);
                }}
              >
                <option value="pi">pi</option>
                <option value="dsh">dsh</option>
              </select>
            </label>
            <label className="cp-field">
              <span>Model (this account&apos;s catalog)</span>
              <select
                name="binding_model"
                value={modelId}
                onChange={(event) => {
                  setModelId(event.target.value);
                  setStale(false);
                }}
              >
                <option value="">Select model</option>
                {models.map((model) => (
                  <option key={model.id} value={model.id}>
                    {model.id}
                  </option>
                ))}
              </select>
            </label>
            {models.length === 0 ? (
              <p className="cp-quiet">
                Catalog is empty — add a model in Models before setting a binding.
              </p>
            ) : null}
            {modelId ? (
              <div data-testid="binding-preview">
                <p>
                  Exact tuple: agent <code className="cp-mono">{agent}</code> → account{" "}
                  <code className="cp-mono">{accountId}</code> → model{" "}
                  <code className="cp-mono">{modelId}</code> · expected revision{" "}
                  <code className="cp-mono">{expected}</code>
                </p>
                {stale ? (
                  <p className="cp-honesty">
                    Changed under you — the revision above was re-read from the daemon after the
                    409. Re-confirm to proceed; nothing was retried automatically.
                  </p>
                ) : null}
                <ConfirmSurface
                  title="Confirm fixed binding"
                  consequences="One active binding per agent; this replaces any current active binding for the agent. No fallback, no per-request override."
                  targets={[
                    `agent: ${agent}`,
                    `account: ${accountId}`,
                    `model: ${modelId}`,
                    `expected_revision: ${expected}`,
                  ]}
                  confirmLabel="Confirm this exact agent, account, model, and revision"
                  actionLabel="Set binding"
                  onConfirm={() => void setBinding()}
                />
              </div>
            ) : null}
          </div>
          <DshApplyBlock
            accountId={accountId}
            bindings={rows}
            models={models}
            onChanged={onChanged}
          />
        </>
      ) : null}
      {/*
       * Set / remove each refresh this projection; the receipt and the reason
       * must outlive that refresh, so they render outside the projection guard.
       */}
      <p role="status">{message}</p>
      {receipt ? <ReceiptLine>{receipt}</ReceiptLine> : null}
    </>
  );
}

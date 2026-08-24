import { useState, type FormEvent } from "react";
import { readJson, rejectCallerHeaderInjection } from "../../api";
import { ConfirmSurface } from "../../components/ConfirmSurface";
import { asRecord } from "../../data/projections";
import type { SecretPresence } from "../../data/projections/providers";
import { classifyProbe } from "../../probe";

/**
 * KeyHandoffForm — docs/design/17 §3 + ADR-0053. The key field is
 * memory-only, non-echoing, cleared on submit (success or failure), and
 * never logged, persisted, or rendered back. `op` is chosen by current
 * presence (set vs rotate); remove revokes the account and its consequence
 * is named before the act.
 */
export function KeyHandoffForm({
  accountId,
  presence,
  onDone,
}: {
  accountId: string;
  presence: SecretPresence;
  onDone?: () => void;
}) {
  const [key, setKey] = useState("");
  const [message, setMessage] = useState(
    "Key field is memory-only and cleared after submit.",
  );
  const op = presence === "present" ? "rotate" : "set";

  async function submit(event: FormEvent) {
    event.preventDefault();
    const apiKey = key;
    setKey("");
    const body = { id: accountId, op, api_key: apiKey };
    rejectCallerHeaderInjection(body);
    const result = await readJson("/management/providers/accounts/key", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    const probe = classifyProbe({
      ok: result.ok,
      httpStatus: result.status,
      body: result.body,
    });
    setMessage(
      result.ok
        ? `Key handed to the daemon SecretStore path (op: ${op}). Probe class ${probe.label}. Response redacted.`
        : `HTTP ${result.status} ${String(asRecord(result.body).code ?? "")} · ${probe.label}`,
    );
    onDone?.();
  }

  async function removeKey() {
    const result = await readJson("/management/providers/accounts/key", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ id: accountId, op: "remove" }),
    });
    setMessage(
      result.ok
        ? "Key removed; account revoked."
        : `HTTP ${result.status} ${String(asRecord(result.body).code ?? "")}`,
    );
    onDone?.();
  }

  return (
    <div className="cp-subblock">
      <h4 className="cp-section-title">Key handoff</h4>
      <form
        onSubmit={(event) => {
          void submit(event);
        }}
      >
        <p className="cp-quiet">
          The key is sent once on the management channel and cleared from this field. SecretRef
          is not a resolvable credential in the browser. Current operation:{" "}
          <code className="cp-mono">{op}</code> (chosen by secret presence).
        </p>
        <label className="cp-field">
          <span>API key</span>
          <input
            type="password"
            autoComplete="off"
            value={key}
            onChange={(event) => setKey(event.target.value)}
          />
        </label>
        <button type="submit" className="cp-button cp-button--primary">
          {op === "rotate" ? "Rotate key via daemon" : "Set key via daemon"}
        </button>
      </form>
      <ConfirmSurface
        title="Remove key"
        consequences="Removing the key revokes this account; bindings become non-callable. Deleting is separate from repairing and is never the suggested recovery."
        targets={[`account: ${accountId}`, "op: remove"]}
        confirmLabel="I understand removing the key revokes this account"
        actionLabel="Remove key"
        danger
        onConfirm={() => void removeKey()}
      />
      <p role="status">{message}</p>
    </div>
  );
}

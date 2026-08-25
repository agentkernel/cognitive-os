import { useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import { readJson, rejectCallerHeaderInjection } from "../../api";
import { ReceiptLine } from "../../components/ReceiptLine";
import { asRecord } from "../../data/projections";
import { PROVIDER_KINDS, requiresTrustConfirmation, type ProviderKind } from "../../probe";

/**
 * AccountCreateFlow — docs/design/17 §3. The documented order:
 * validate → trust confirmation (when the endpoint grant requires it) →
 * persist account → key handoff on the account page → SecretStore write →
 * bounded probe → verify. The browser never writes SecretStore and never
 * calls the Provider; a created-but-keyless account sits revoked by design.
 */
export function AccountCreateFlow({ onCreated }: { onCreated?: () => void }) {
  const [kind, setKind] = useState<ProviderKind>("openai_official");
  const [allowPrivate, setAllowPrivate] = useState(false);
  const [allowInsecure, setAllowInsecure] = useState(false);
  const [trustConfirmed, setTrustConfirmed] = useState(false);
  const [message, setMessage] = useState(
    "Keys travel only in the key POST body, then SecretStore.",
  );
  const [createdId, setCreatedId] = useState<string | undefined>();

  const needsTrust = requiresTrustConfirmation({
    kind,
    allowPrivateNetwork: allowPrivate,
    allowInsecureHttp: allowInsecure,
  });

  async function create(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = event.currentTarget;
    if (needsTrust && !trustConfirmed) {
      setCreatedId(undefined);
      setMessage(
        "Trust confirmation is required before persisting a private or HTTP endpoint.",
      );
      return;
    }
    const data = new FormData(form);
    const body = {
      display_name: String(data.get("display_name") ?? ""),
      provider_kind: kind,
      endpoint: String(data.get("endpoint") ?? "") || undefined,
      allow_private_network: allowPrivate,
      allow_insecure_http: allowInsecure,
    };
    rejectCallerHeaderInjection(body);
    const result = await readJson("/management/providers/accounts", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    if (result.ok) {
      const created = asRecord(asRecord(result.body).account);
      const id = String(created.id ?? "unknown");
      setCreatedId(id);
      setMessage("Account persisted. The key handoff and bounded probe happen on the account page.");
      form.reset();
      setKind("openai_official");
      setAllowPrivate(false);
      setAllowInsecure(false);
      setTrustConfirmed(false);
      onCreated?.();
    } else {
      setCreatedId(undefined);
      setMessage(`HTTP ${result.status} ${String(asRecord(result.body).code ?? "")}`);
    }
  }

  return (
    <section className="cp-panel" aria-labelledby="create-account-heading">
      <h3 id="create-account-heading" className="cp-section-title">
        Create named account
      </h3>
      <form
        onSubmit={(event) => {
          void create(event);
        }}
      >
        <p className="cp-quiet">
          Sequence: validate → trust confirmation when required → persist account → key handoff
          on the account page → SecretStore write → bounded probe → verify. The browser does not
          write SecretStore and does not call the Provider.
        </p>
        <label className="cp-field">
          <span>Display name</span>
          <input name="display_name" required />
        </label>
        <label className="cp-field">
          <span>Kind</span>
          <select
            name="provider_kind"
            required
            value={kind}
            onChange={(event) => setKind(event.target.value as ProviderKind)}
          >
            {PROVIDER_KINDS.map((item) => (
              <option key={item} value={item}>
                {item}
              </option>
            ))}
          </select>
        </label>
        <label className="cp-field">
          <span>Endpoint</span>
          <input name="endpoint" placeholder="only for openai_compatible" />
        </label>
        <label className="cp-field">
          <input
            type="checkbox"
            name="allow_private_network"
            checked={allowPrivate}
            onChange={(event) => setAllowPrivate(event.target.checked)}
          />{" "}
          Allow private network
        </label>
        <label className="cp-field">
          <input
            type="checkbox"
            name="allow_insecure_http"
            checked={allowInsecure}
            onChange={(event) => setAllowInsecure(event.target.checked)}
          />{" "}
          Allow insecure HTTP
        </label>
        {needsTrust ? (
          <label className="cp-field">
            <input
              type="checkbox"
              name="trust_confirmed"
              checked={trustConfirmed}
              onChange={(event) => setTrustConfirmed(event.target.checked)}
            />{" "}
            I confirm this private-network or HTTP endpoint grant
          </label>
        ) : null}
        <button type="submit" className="cp-button cp-button--primary">
          Create account
        </button>
      </form>
      <p role="status">{message}</p>
      {createdId ? (
        <ReceiptLine>
          Account <span className="cp-mono">{createdId}</span> created. Next:{" "}
          <Link to={`/providers/${encodeURIComponent(createdId)}`}>
            enter the API key on the account page
          </Link>{" "}
          — it is not in this form — then run the bounded probe to verify.
        </ReceiptLine>
      ) : null}
    </section>
  );
}

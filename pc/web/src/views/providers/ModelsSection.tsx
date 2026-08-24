import { useState, type FormEvent } from "react";
import { readJson } from "../../api";
import { ReceiptLine } from "../../components/ReceiptLine";
import { asRecord } from "../../data/projections";
import type { ProviderModel } from "../../data/projections/providers";
import type { Projection } from "../../data/store";
import { displayCost } from "../../policy";
import { capabilityDisposition, classifyProbe } from "../../probe";
import { ProjectionState } from "./ProjectionState";

/**
 * Models section — docs/design/17 §2.2. Catalog with source honesty
 * (discovered vs manual — manual visibly less certain), pricing via
 * policy.displayCost (cost_unavailable/unknown never render as 0). Refresh
 * is an explicit bounded probe; a failed refresh preserves the last catalog
 * and says so.
 */
export function ModelsSection({
  accountId,
  models,
  onChanged,
}: {
  accountId: string;
  models: Projection<ProviderModel[]>;
  onChanged: () => void;
}) {
  const [message, setMessage] = useState(
    "Failed refresh keeps the last catalog. Unknown prices are never shown as zero.",
  );
  const [receipt, setReceipt] = useState<string | undefined>();
  const [probing, setProbing] = useState(false);

  async function probe() {
    setProbing(true);
    setReceipt(undefined);
    const result = await readJson("/management/providers/models/refresh", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ id: accountId }),
    });
    setProbing(false);
    const classified = classifyProbe({
      ok: result.ok,
      httpStatus: result.status,
      body: result.body,
    });
    setMessage(
      result.ok
        ? `Reachability not implied. Model discovery ${classified.label} in ${result.ms} ms. Capability ${capabilityDisposition(undefined)}.`
        : `Probe failed (HTTP ${result.status} · ${classified.label}). The last catalog and bindings are preserved. ${classified.nextAction}`,
    );
    onChanged();
  }

  async function addModel(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = event.currentTarget;
    const modelId = String(new FormData(form).get("model_id") ?? "");
    const result = await readJson("/management/providers/models/add", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ account_id: accountId, model_id: modelId }),
    });
    if (result.ok) {
      setReceipt(
        `Manual model ${modelId} stored. Manual entries are less certain than discovered ones.`,
      );
      setMessage("Failed refresh keeps the last catalog.");
      form.reset();
    } else {
      setReceipt(undefined);
      setMessage(`HTTP ${result.status} ${String(asRecord(result.body).code ?? "")}`);
    }
    onChanged();
  }

  async function setPrice(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = event.currentTarget;
    const data = new FormData(form);
    const modelId = String(data.get("price_model_id") ?? "");
    const input = String(data.get("price_input") ?? "").trim();
    const output = String(data.get("price_output") ?? "").trim();
    const body: Record<string, string> = { account_id: accountId, model_id: modelId };
    if (input) {
      body.price_input_per_million = input;
    }
    if (output) {
      body.price_output_per_million = output;
    }
    const result = await readJson("/management/providers/models/set-price", "management", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    if (result.ok) {
      setReceipt(`Price stored for ${modelId}.`);
      setMessage("Failed refresh keeps the last catalog.");
      form.reset();
    } else {
      setReceipt(undefined);
      setMessage(`HTTP ${result.status} ${String(asRecord(result.body).code ?? "")}`);
    }
    onChanged();
  }

  const rows = models.data ?? [];

  return (
    <>
      <ProjectionState projection={models} what="Model catalog" />
      {models.status === "ready" || models.status === "empty" || models.status === "stale" ? (
        <>
          {rows.length === 0 ? (
            <p className="cp-quiet">
              Catalog is empty (authoritative). Add a model manually or run the bounded probe.
            </p>
          ) : (
            <table className="cp-table">
              <caption>
                Catalog — failed refresh keeps the last catalog; unknown prices are never 0
              </caption>
              <thead>
                <tr>
                  <th scope="col">Model</th>
                  <th scope="col">Source</th>
                  <th scope="col">Input $/M</th>
                  <th scope="col">Output $/M</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((model) => (
                  <tr key={model.id}>
                    <td className="cp-mono">{model.id}</td>
                    <td>
                      <span className="cp-mono">{model.source}</span>
                      {model.source === "manual" ? (
                        <span className="cp-quiet"> (manual — less certain)</span>
                      ) : null}
                    </td>
                    <td className="cp-mono">{displayCost(model.priceInputPerMillion)}</td>
                    <td className="cp-mono">{displayCost(model.priceOutputPerMillion)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
          <p>
            <button
              type="button"
              className="cp-button"
              disabled={probing}
              onClick={() => void probe()}
            >
              Refresh catalog (bounded probe)
            </button>
          </p>
          <form
            onSubmit={(event) => {
              void addModel(event);
            }}
          >
            <h4 className="cp-section-title">Add model manually</h4>
            <p className="cp-quiet">
              Use this when discovery is degraded. Endpoint-servability is enforced by the
              daemon. Do not display unknown or cost_unavailable as zero or ready.
            </p>
            <label className="cp-field">
              <span>Model id</span>
              <input name="model_id" required placeholder="deepseek-chat" />
            </label>
            <button type="submit" className="cp-button">
              Add model
            </button>
          </form>
          <form
            onSubmit={(event) => {
              void setPrice(event);
            }}
          >
            <h4 className="cp-section-title">Set price</h4>
            <p className="cp-quiet">
              Prices are per million tokens, stored verbatim. Empty fields are not sent.
            </p>
            <label className="cp-field">
              <span>Model id</span>
              <input name="price_model_id" required placeholder="deepseek-chat" />
            </label>
            <label className="cp-field">
              <span>Input $/M</span>
              <input name="price_input" inputMode="decimal" placeholder="0.27" />
            </label>
            <label className="cp-field">
              <span>Output $/M</span>
              <input name="price_output" inputMode="decimal" placeholder="1.10" />
            </label>
            <button type="submit" className="cp-button">
              Set price
            </button>
          </form>
        </>
      ) : null}
      {/*
       * Probe / add / set-price each refresh this projection; their outcome
       * must outlive that refresh, so it renders outside the projection guard.
       */}
      <p role="status">{message}</p>
      {receipt ? <ReceiptLine>{receipt}</ReceiptLine> : null}
    </>
  );
}

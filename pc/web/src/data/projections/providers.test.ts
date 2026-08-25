import { describe, expect, it } from "vitest";
import { bindingRevisionForCas, displayCost } from "../../policy";
import {
  projectBindings,
  projectBudgets,
  projectProviderAccountDetail,
  projectProviderAlerts,
  projectProviderModels,
  projectAuditEvents,
  projectUsageEvents,
  triageAccounts,
  usageCostLabel,
  type ProviderAccount,
} from "./providers";

describe("provider projections — triage and honesty", () => {
  it("orders accounts attention-first with a deterministic tiebreak", () => {
    const accounts: ProviderAccount[] = [
      { id: "a3", name: "zed", kind: "k", status: "active", secret: "present" },
      { id: "a1", name: "beta", kind: "k", status: "revoked", secret: "absent" },
      { id: "a2", name: "alpha", kind: "k", status: "revoked", secret: "absent" },
      { id: "a4", name: "mid", kind: "k", status: "degraded", secret: "unknown" },
      { id: "a5", name: "mystery", kind: "k", status: "unmapped-word", secret: "unknown" },
    ];
    const sorted = triageAccounts(accounts);
    // revoked (name: alpha, beta) → degraded → active → unmapped last
    expect(sorted.map((account) => account.id)).toEqual(["a2", "a1", "a4", "a3", "a5"]);
    // input array is not mutated
    expect(accounts[0].id).toBe("a3");
  });

  it("keeps unknown model prices unknown — never 0, never fabricated", () => {
    const models = projectProviderModels({
      models: [
        {
          account_id: "a",
          model_id: "m1",
          source: "discovered",
          pricing_version: "v1",
          price_input_per_million: "0.27",
          price_output_per_million: "1.10",
        },
        {
          account_id: "a",
          model_id: "m2",
          source: "manual",
          price_input_per_million: null,
          price_output_per_million: null,
        },
      ],
    });
    expect(models[0].priceInputPerMillion).toBe("0.27");
    expect(models[0].source).toBe("discovered");
    expect(models[1].priceInputPerMillion).toBeUndefined();
    expect(models[1].priceOutputPerMillion).toBeUndefined();
    expect(displayCost(models[1].priceInputPerMillion)).toBe("unknown");
    expect(displayCost(undefined, "cost_unavailable")).toBe("cost_unavailable");
  });

  it("preserves binding revision verbatim and never fabricates one", () => {
    const bindings = projectBindings({
      bindings: [
        { agent: "pi", account_id: "a", model_id: "m", revision: 4, status: "active" },
        { agent: "dsh", account_id: "a", model_id: "m", status: "revoked" },
      ],
    });
    expect(bindings[0].revision).toBe(4);
    expect(bindings[1].revision).toBeUndefined();
    // Active-only CAS: active supplies its revision, revoked supplies 0.
    expect(bindingRevisionForCas(bindings[0])).toBe(4);
    expect(bindingRevisionForCas(bindings[1])).toBe(0);
  });

  it("keeps cost_unavailable usage events free of fabricated zeros", () => {
    const events = projectUsageEvents({
      events: [
        { event_id: "e1", account_id: "a", cost_micros: null, cost_status: "cost_unavailable" },
        { event_id: "e2", account_id: "a", cost_micros: 1940000, cost_status: "priced" },
      ],
    });
    expect(events[0].costMicros).toBeUndefined();
    expect(usageCostLabel(events[0])).toBe("cost_unavailable");
    expect(usageCostLabel(events[1])).toBe("$1.940000");
    expect(usageCostLabel({ id: "e3", accountId: "a", costStatus: "unknown" })).toBe("unknown");
  });

  it("projects budgets, alerts, and audit rows without inventing fields", () => {
    const budgets = projectBudgets({
      budgets: [
        {
          budget_id: "b1",
          scope_kind: "account",
          scope_id: "a",
          token_limit: 100,
          amount_micros_limit: 5000000,
        },
      ],
    });
    expect(budgets[0]).toEqual({
      id: "b1",
      scopeKind: "account",
      scopeId: "a",
      tokenLimit: 100,
      amountMicrosLimit: 5000000,
    });
    const alerts = projectProviderAlerts({
      alerts: [
        {
          alert_id: "al-1",
          budget_id: "b1",
          threshold_kind: "warning_80",
          issued_at_ms: 7,
          acknowledged_at_ms: null,
        },
      ],
    });
    expect(alerts[0]).toEqual({
      id: "al-1",
      budgetId: "b1",
      threshold: "warning_80",
      issuedAtMs: 7,
      acknowledged: false,
    });
    const audit = projectAuditEvents({
      events: [{ audit_id: "au-1", action: "account.created", outcome: "ok", detail: "a" }],
    });
    expect(audit[0].action).toBe("account.created");
    expect(audit[0].detail).toBe("a");
  });

  it("projects account detail with presence-only secrets and trust facts", () => {
    const detail = projectProviderAccountDetail({
      account: {
        id: "a",
        display_name: "n",
        provider_kind: "openai_compatible",
        status: "active",
        secret_ref: "ss://provider/a",
        endpoint: "https://e/v1",
        allow_private_network: true,
        allow_insecure_http: false,
        catalog_revision: 3,
        network_scope: "private",
      },
    });
    expect(detail.secret).toBe("present");
    expect(JSON.stringify(detail)).not.toContain("ss://");
    expect(detail.endpoint).toBe("https://e/v1");
    expect(detail.allowPrivateNetwork).toBe(true);
    expect(detail.allowInsecureHttp).toBe(false);
    expect(detail.catalogRevision).toBe("3");
  });
});

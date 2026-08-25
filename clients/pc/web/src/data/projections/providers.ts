/*
 * Provider-plane typed view models (W2) — docs/design/17.
 *
 * Every projection carries unknown-as-unknown: missing fields stay
 * undefined, `cost_unavailable` stays text, `secret_ref` becomes presence
 * only (never a value, never a masked fragment). Views never see raw
 * backend JSON; the fetch pipeline has already normalized envelopes and
 * redacted secret-shaped values at the boundary.
 */

import {
  asList,
  asRecord,
  projectProviderAccount,
  projectProviderAccounts,
  type ProviderAccountView,
  type SecretPresence,
} from "../projections";

export type ProviderAccount = ProviderAccountView;
export type { SecretPresence };
export { projectProviderAccount, projectProviderAccounts };

/**
 * Triage ordering (docs/design/17 §1): revoked/degraded float above active,
 * then name, then id — the list itself is the triage. Unmapped status words
 * sort last but keep their verbatim label (the chip renders them unmapped).
 * Stable: Array.prototype.sort is stable and the tiebreak is total.
 */
export function triageAccounts(accounts: ProviderAccount[]): ProviderAccount[] {
  const rank = (status: string): number => {
    if (status === "revoked") {
      return 0;
    }
    if (status === "degraded") {
      return 1;
    }
    if (status === "active") {
      return 2;
    }
    return 3;
  };
  return [...accounts].sort(
    (a, b) =>
      rank(a.status) - rank(b.status) ||
      a.name.localeCompare(b.name) ||
      a.id.localeCompare(b.id),
  );
}

/* ---------- account detail (inspect) ---------- */

export interface ProviderAccountDetail extends ProviderAccountView {
  endpoint?: string;
  allowPrivateNetwork?: boolean;
  allowInsecureHttp?: boolean;
}

export function projectProviderAccountDetail(body: unknown): ProviderAccountDetail {
  const record = asRecord(asRecord(body).account ?? body);
  return {
    ...projectProviderAccount(record),
    endpoint: record.endpoint == null ? undefined : String(record.endpoint),
    allowPrivateNetwork:
      typeof record.allow_private_network === "boolean"
        ? record.allow_private_network
        : undefined,
    allowInsecureHttp:
      typeof record.allow_insecure_http === "boolean" ? record.allow_insecure_http : undefined,
  };
}

/* ---------- models ---------- */

export interface ProviderModel {
  id: string;
  accountId: string;
  /** Verbatim daemon source ("discovered" | "manual" | …); manual is less certain. */
  source: string;
  pricingVersion?: string;
  /** Price strings verbatim; undefined when the daemon does not provide one. */
  priceInputPerMillion?: string;
  priceOutputPerMillion?: string;
  priceCacheReadPerMillion?: string;
  priceCacheWritePerMillion?: string;
}

export function projectProviderModels(body: unknown): ProviderModel[] {
  return asList(body, ["models", "items"]).map((row) => {
    const record = asRecord(row);
    return {
      id: String(record.model_id ?? "unknown"),
      accountId: String(record.account_id ?? "unknown"),
      source: String(record.source ?? "unknown"),
      pricingVersion:
        record.pricing_version == null ? undefined : String(record.pricing_version),
      priceInputPerMillion:
        record.price_input_per_million == null
          ? undefined
          : String(record.price_input_per_million),
      priceOutputPerMillion:
        record.price_output_per_million == null
          ? undefined
          : String(record.price_output_per_million),
      priceCacheReadPerMillion:
        record.price_cache_read_per_million == null
          ? undefined
          : String(record.price_cache_read_per_million),
      priceCacheWritePerMillion:
        record.price_cache_write_per_million == null
          ? undefined
          : String(record.price_cache_write_per_million),
    };
  });
}

/* ---------- bindings ---------- */

export interface BindingView {
  agent: string;
  accountId: string;
  modelId: string;
  /** Verbatim daemon revision; undefined when absent (never fabricated). */
  revision?: number;
  status: string;
}

export function projectBindings(body: unknown): BindingView[] {
  return asList(body, ["bindings", "items"]).map((row) => {
    const record = asRecord(row);
    const revision = Number(record.revision);
    return {
      agent: String(record.agent ?? "unknown"),
      accountId: String(record.account_id ?? "unknown"),
      modelId: String(record.model_id ?? "unknown"),
      revision:
        record.revision != null && Number.isFinite(revision) ? revision : undefined,
      status: String(record.status ?? "unknown"),
    };
  });
}

/* ---------- usage / budgets / alerts / audit ---------- */

export interface UsageEventView {
  id: string;
  accountId: string;
  /** Micro-dollars; undefined when cost is unavailable. Never defaulted to 0. */
  costMicros?: number;
  /** Verbatim daemon vocabulary: "priced" | "cost_unavailable" | … */
  costStatus: string;
}

export function projectUsageEvents(body: unknown): UsageEventView[] {
  return asList(body, ["events", "items"]).map((row) => {
    const record = asRecord(row);
    const cost = Number(record.cost_micros);
    return {
      id: String(record.event_id ?? "unknown"),
      accountId: String(record.account_id ?? "unknown"),
      costMicros:
        record.cost_micros == null || !Number.isFinite(cost) ? undefined : cost,
      costStatus: String(record.cost_status ?? "unknown"),
    };
  });
}

/**
 * Usage cost cell: priced → dollars from micros; cost_unavailable/unknown
 * stay text. Complements policy.displayCost (model prices) — unknown is
 * never rendered as 0 in either place.
 */
export function usageCostLabel(event: UsageEventView): string {
  if (event.costStatus === "cost_unavailable") {
    return "cost_unavailable";
  }
  if (event.costMicros == null) {
    return "unknown";
  }
  return `$${(event.costMicros / 1_000_000).toFixed(6)}`;
}

export interface BudgetView {
  id: string;
  scopeKind: string;
  scopeId: string;
  tokenLimit?: number;
  amountMicrosLimit?: number;
}

export function projectBudgets(body: unknown): BudgetView[] {
  return asList(body, ["budgets", "items"]).map((row) => {
    const record = asRecord(row);
    const tokens = Number(record.token_limit);
    const amount = Number(record.amount_micros_limit);
    return {
      id: String(record.budget_id ?? "unknown"),
      scopeKind: String(record.scope_kind ?? "unknown"),
      scopeId: String(record.scope_id ?? "unknown"),
      tokenLimit:
        record.token_limit == null || !Number.isFinite(tokens) ? undefined : tokens,
      amountMicrosLimit:
        record.amount_micros_limit == null || !Number.isFinite(amount) ? undefined : amount,
    };
  });
}

export interface ProviderAlertView {
  id: string;
  /** Alerts link to an account only through their budget's scope. */
  budgetId?: string;
  threshold: string;
  issuedAtMs?: number;
  acknowledged: boolean;
}

export function projectProviderAlerts(body: unknown): ProviderAlertView[] {
  return asList(body, ["alerts", "events", "items"]).map((row) => {
    const record = asRecord(row);
    return {
      id: String(record.alert_id ?? record.id ?? "unknown"),
      budgetId: record.budget_id == null ? undefined : String(record.budget_id),
      threshold: String(record.threshold_kind ?? record.kind ?? "unknown"),
      issuedAtMs:
        typeof record.issued_at_ms === "number" ? record.issued_at_ms : undefined,
      acknowledged: record.acknowledged_at_ms != null || record.acknowledged === true,
    };
  });
}

export interface AuditEventView {
  id: string;
  action: string;
  outcome: string;
  detail?: string;
}

export function projectAuditEvents(body: unknown): AuditEventView[] {
  return asList(body, ["events", "items"]).map((row) => {
    const record = asRecord(row);
    return {
      id: String(record.audit_id ?? record.id ?? "unknown"),
      action: String(record.action ?? "unknown"),
      outcome: String(record.outcome ?? "unknown"),
      detail: record.detail == null ? undefined : String(record.detail),
    };
  });
}

/* ---------- dsh runtime (observation only; candidate_only per contract) ---------- */

export const DSH_RUNTIME_KEY = "dsh:runtime";
export const DSH_SELECTED_KEY = "dsh:selected";

export interface DshSessionView {
  sessionId: string;
  state?: string;
  fencingEpoch?: number;
  lastSequence?: number;
  taskRef?: string;
}

export interface DshRuntimeView {
  state?: string;
  processAlive?: boolean;
  processId?: number;
  sessionCount?: number;
  sessions: DshSessionView[];
  lastHeartbeatUnixMs?: number;
  candidateOnly?: boolean;
  dshResponseIsNotTaskCompletion?: boolean;
}

export function projectDshRuntime(body: unknown): DshRuntimeView {
  const record = asRecord(body);
  const heartbeat = Number(record.last_heartbeat_unix_ms);
  const processId = Number(record.process_id);
  const sessionCount = Number(record.session_count);
  return {
    state: record.state == null ? undefined : String(record.state),
    processAlive:
      typeof record.process_alive === "boolean" ? record.process_alive : undefined,
    processId:
      record.process_id == null || !Number.isFinite(processId) ? undefined : processId,
    sessionCount:
      record.session_count == null || !Number.isFinite(sessionCount)
        ? undefined
        : sessionCount,
    sessions: asList(body, ["sessions"]).map((row) => {
      const session = asRecord(row);
      const epoch = Number(session.fencing_epoch);
      const sequence = Number(session.last_sequence);
      return {
        sessionId: String(session.session_id ?? "unknown"),
        state: session.state == null ? undefined : String(session.state),
        fencingEpoch:
          session.fencing_epoch == null || !Number.isFinite(epoch) ? undefined : epoch,
        lastSequence:
          session.last_sequence == null || !Number.isFinite(sequence)
            ? undefined
            : sequence,
        taskRef:
          session.task_ref == null || session.task_ref === ""
            ? undefined
            : String(session.task_ref),
      };
    }),
    lastHeartbeatUnixMs:
      record.last_heartbeat_unix_ms == null || !Number.isFinite(heartbeat)
        ? undefined
        : heartbeat,
    candidateOnly: record.candidate_only === true ? true : undefined,
    dshResponseIsNotTaskCompletion:
      record.dsh_response_is_not_task_completion === true ? true : undefined,
  };
}

export interface DshSelectedView {
  selectedModel?: string;
}

export function projectDshSelected(body: unknown): DshSelectedView {
  const record = asRecord(body);
  return {
    selectedModel:
      record.selected_model == null ? undefined : String(record.selected_model),
  };
}

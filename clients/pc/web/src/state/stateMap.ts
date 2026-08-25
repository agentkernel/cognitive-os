/*
 * State system — docs/design/22-control-plane-state-system.md.
 * Two layers: display CATEGORY (7, visual semantics) × verbatim domain LABEL.
 * The UI never mints authority states; unmapped words degrade to
 * category "unknown" with unmapped:true and the verbatim label preserved.
 */

export type StateCategory =
  | "ready"
  | "active"
  | "waiting"
  | "attention"
  | "blocked"
  | "completed"
  | "unknown";

export interface StateReading {
  category: StateCategory;
  /** The daemon's own state word, verbatim (or a client-honest word like "not-run"). */
  label: string;
  /** True when the raw word was not present in the known table — never guess a color. */
  unmapped: boolean;
}

export const CATEGORY_META: Record<
  StateCategory,
  { word: string; shape: string; note: string }
> = {
  ready: { word: "Ready", shape: "filled-circle", note: "nominal; nothing needed" },
  active: { word: "Active", shape: "pulse-circle", note: "work in flight / live now" },
  waiting: { word: "Waiting", shape: "hollow-dot", note: "paused on a precondition" },
  attention: { word: "Attention", shape: "diamond", note: "degraded; act soon" },
  blocked: { word: "Blocked", shape: "square", note: "cannot proceed; act now" },
  completed: { word: "Completed", shape: "check", note: "terminally done — evidence-linked" },
  unknown: { word: "Unavailable", shape: "hollow-circle", note: "fact unavailable / unknown / not-run" },
};

/* Verified daemon vocabularies (specs/transitions/*.json; kernel-server personal handlers). */

/** specs/transitions/task.transitions.json — 9 states. */
export const TASK_STATE_CATEGORY: Record<string, StateCategory> = {
  DRAFT: "waiting",
  READY: "waiting",
  ACTIVE: "active",
  BLOCKED: "blocked",
  CANDIDATE_COMPLETE: "waiting",
  COMPLETED: "completed",
  FAILED: "blocked",
  CANCELLED: "blocked",
  ESCALATED: "blocked",
};

/** specs/transitions/effect.transitions.json — 14 states. */
export const EFFECT_STAGE_CATEGORY: Record<string, StateCategory> = {
  PROPOSED: "waiting",
  AUTHORIZED: "waiting",
  DENIED: "blocked",
  EXECUTING: "active",
  EXECUTED: "ready",
  OUTCOME_UNKNOWN: "blocked",
  RECONCILED: "ready",
  VERIFIED: "ready",
  VERIFY_FAILED: "blocked",
  COMPENSATING: "waiting",
  NOT_EXECUTED: "unknown",
  COMMITTED: "ready",
  ABORTED: "blocked",
  QUARANTINED: "blocked",
};

/** effect outcome/reconcile classes (read-side projection, task_api.rs). */
export const EFFECT_OUTCOME_CATEGORY: Record<string, StateCategory> = {
  executed: "ready",
  not_executed: "unknown",
  failed: "blocked",
  indeterminate: "unknown",
};

export const EFFECT_RECONCILE_CATEGORY: Record<string, StateCategory> = {
  closed: "ready",
  not_applicable: "unknown",
  pending_reconciliation: "waiting",
  must_reconcile: "attention",
};

/** readiness.rs — overall blocked|degraded|ready; components + not_configured. */
export const READINESS_CATEGORY: Record<string, StateCategory> = {
  ready: "ready",
  degraded: "attention",
  blocked: "blocked",
  not_configured: "unknown",
};

/** provider_control_plane.rs — account status. */
export const PROVIDER_STATUS_CATEGORY: Record<string, StateCategory> = {
  active: "ready",
  degraded: "attention",
  revoked: "blocked",
};

/** Provider key presence (secret_ref is never displayed — presence only). */
export const SECRET_PRESENCE_CATEGORY: Record<string, StateCategory> = {
  present: "ready",
  absent: "blocked",
  unknown: "unknown",
};

/** tool_lifecycle.rs overlay. */
export const TOOL_LIFECYCLE_CATEGORY: Record<string, StateCategory> = {
  enabled: "ready",
  disabled: "unknown",
  quarantined: "blocked",
  revoked: "unknown",
};

/** dsh runtime snapshot (task_api.rs). */
export const DSH_RUNTIME_CATEGORY: Record<string, StateCategory> = {
  ACTIVE: "active",
  INACTIVE: "unknown",
  CRASHED: "blocked",
};

/** watch controller (watch.ts client state machine). */
export const WATCH_STATE_CATEGORY: Record<string, StateCategory> = {
  live: "active",
  stale: "attention",
  disconnected: "blocked",
  reconciling: "waiting",
  unknown: "unknown",
};

/** Client load/display states (not authority states — display vocabulary). */
export const LOAD_STATE_CATEGORY: Record<string, StateCategory> = {
  loading: "unknown",
  ready: "ready",
  empty: "unknown",
  denied: "blocked",
  disconnected: "blocked",
  unknown: "unknown",
  stale: "attention",
  "not-run": "unknown",
};

const TABLES: Record<string, Record<string, StateCategory>> = {
  task: TASK_STATE_CATEGORY,
  effect: EFFECT_STAGE_CATEGORY,
  effect_outcome: EFFECT_OUTCOME_CATEGORY,
  effect_reconcile: EFFECT_RECONCILE_CATEGORY,
  readiness: READINESS_CATEGORY,
  provider: PROVIDER_STATUS_CATEGORY,
  secret: SECRET_PRESENCE_CATEGORY,
  tool: TOOL_LIFECYCLE_CATEGORY,
  dsh: DSH_RUNTIME_CATEGORY,
  watch: WATCH_STATE_CATEGORY,
  load: LOAD_STATE_CATEGORY,
};

export type StateDomain = keyof typeof TABLES;

/**
 * Map one verbatim domain state word onto a display category.
 * Empty/missing → unknown ("unknown"). Unknown word → unknown + unmapped.
 */
export function readDomainState(domain: StateDomain, raw: unknown): StateReading {
  const table = TABLES[domain];
  if (raw == null) {
    return { category: "unknown", label: "unknown", unmapped: false };
  }
  const label = String(raw).trim();
  if (label === "") {
    return { category: "unknown", label: "unknown", unmapped: false };
  }
  const hit = table[label] ?? table[label.toLowerCase()];
  if (hit) {
    return { category: hit, label, unmapped: false };
  }
  return { category: "unknown", label, unmapped: true };
}

/** Binding state needs dispatchability, not just status (docs/design/22 §3). */
export function readBindingState(status: unknown, dispatchable: boolean | undefined): StateReading {
  const base = String(status ?? "").trim();
  if (base === "active") {
    if (dispatchable === true) {
      return { category: "ready", label: "active · callable", unmapped: false };
    }
    if (dispatchable === false) {
      return { category: "blocked", label: "active · blocked", unmapped: false };
    }
    return { category: "attention", label: "active · dispatch unknown", unmapped: false };
  }
  if (base === "revoked") {
    return { category: "unknown", label: "revoked", unmapped: false };
  }
  return readDomainState("provider", base);
}

/**
 * Registered error codes and contract-driven retry classification
 * (docs/standards/error-contract.md §2/§3; REQ-ERR-001, REQ-ERR-002).
 *
 * `REGISTERED_ERRORS` is a pinned copy of `specs/registry/errors.yaml`
 * (v0.1, 55 codes). Contracts codegen does not yet emit an errors-registry
 * binding, so the copy lives here guarded by `errors.test.ts`, which reads
 * the YAML at test time and fails on any drift (code set, category,
 * retryable). Gap registered for Lane-CTR in the 20260720 lane-tsc handoff.
 */

/** Error categories, mirroring the generated `commonDefs.ErrorCategory`. */
export interface RegisteredError {
  readonly category: string;
  readonly retryable: boolean;
}

/** Pinned registry copy: code → {category, retryable}. Drift-gated by test. */
export const REGISTERED_ERRORS: Readonly<Record<string, RegisteredError>> = {
  STATE_CONFLICT: { category: "state", retryable: true },
  STATE_STALE_OBSERVATION: { category: "state", retryable: true },
  STATE_STORE_UNAVAILABLE: { category: "state", retryable: true },
  CONTEXT_INCOMPLETE: { category: "context", retryable: true },
  CONTEXT_BUDGET_EXCEEDED: { category: "context", retryable: false },
  CONTEXT_AUTH_DENIED: { category: "auth", retryable: false },
  AUTH_CAPABILITY_ATTENUATION_VIOLATION: { category: "auth", retryable: false },
  AUTH_CAPABILITY_EXPIRED: { category: "auth", retryable: true },
  EFFECT_OUTCOME_UNKNOWN: { category: "effect", retryable: true },
  EFFECT_RECOVERY_QUARANTINED: { category: "effect", retryable: false },
  EFFECT_IDEMPOTENCY_CONFLICT: { category: "effect", retryable: false },
  PROTOCOL_MAPPING_INCOMPLETE: { category: "protocol", retryable: false },
  PROTOCOL_SCHEMA_DIGEST_MISMATCH: { category: "protocol", retryable: true },
  VERSION_UNSUPPORTED: { category: "protocol", retryable: false },
  CRITICAL_EXTENSION_UNKNOWN: { category: "protocol", retryable: false },
  SCHEMA_MISMATCH: { category: "protocol", retryable: true },
  DIGEST_MISMATCH: { category: "protocol", retryable: false },
  RESOURCE_BUDGET_EXHAUSTED: { category: "resource", retryable: false },
  PROFILE_CIM_CALIBRATION_MISMATCH: { category: "profile", retryable: true },
  PROFILE_LEARNING_PROMOTION_DENIED: { category: "profile", retryable: false },
  PROFILE_EMBODIED_OBSERVATION_STALE: { category: "profile", retryable: true },
  KNOWLEDGE_SOURCE_INVALIDATED: { category: "knowledge", retryable: true },
  KNOWLEDGE_POISON_QUARANTINED: { category: "knowledge", retryable: false },
  KNOWLEDGE_MAINTENANCE_BOUNDED: { category: "knowledge", retryable: false },
  PERFORMANCE_REPORT_INCOMPLETE: { category: "performance", retryable: false },
  MANAGEMENT_SESSION_EXPIRED: { category: "auth", retryable: true },
  MANAGEMENT_SESSION_REVOKED: { category: "auth", retryable: false },
  MANAGEMENT_STEP_UP_REQUIRED: { category: "auth", retryable: true },
  MANAGEMENT_SCOPE_MISMATCH: { category: "auth", retryable: false },
  MANAGEMENT_SELF_AUTHORIZATION_DENIED: { category: "auth", retryable: false },
  MANAGEMENT_INDEPENDENT_APPROVAL_REQUIRED: { category: "auth", retryable: true },
  AGENT_PACKAGE_VERIFICATION_FAILED: { category: "agent", retryable: false },
  AGENT_ADAPTER_BYPASS_DETECTED: { category: "agent", retryable: false },
  AGENT_COMPATIBILITY_DEGRADED: { category: "agent", retryable: false },
  MEMORY_ADMISSION_DENIED: { category: "memory", retryable: false },
  MEMORY_SCOPE_PROMOTION_REQUIRED: { category: "memory", retryable: false },
  MEMORY_DERIVATION_INVALIDATED: { category: "memory", retryable: true },
  RESOURCE_NOT_DISCOVERABLE: { category: "discovery", retryable: false },
  CONTEXT_RESOLUTION_STAGNATED: { category: "discovery", retryable: false },
  CATALOG_VERSION_STALE: { category: "catalog", retryable: true },
  CATALOG_MATCH_INCONCLUSIVE: { category: "catalog", retryable: false },
  NO_AUTHORIZED_OPERATION_CANDIDATE: { category: "catalog", retryable: false },
  SEMANTIC_SERVICE_UNAVAILABLE: { category: "semantic", retryable: true },
  SEMANTIC_MATCH_INCONCLUSIVE: { category: "semantic", retryable: false },
  MODEL_EGRESS_DENIED: { category: "semantic", retryable: false },
  SEMANTIC_BUDGET_EXHAUSTED: { category: "semantic", retryable: false },
  SHELL_TARGET_AMBIGUOUS: { category: "shell", retryable: true },
  SHELL_TARGET_NOT_FOUND: { category: "shell", retryable: true },
  SHELL_PREVIEW_STALE: { category: "shell", retryable: true },
  INTENT_CLARIFICATION_REQUIRED: { category: "intent", retryable: true },
  INTENT_VERSION_SUPERSEDED: { category: "intent", retryable: true },
  CANCEL_PENDING: { category: "lifecycle", retryable: true },
  CANCEL_TOO_LATE: { category: "lifecycle", retryable: false },
  WATCH_CURSOR_STALE: { category: "watch", retryable: true },
  SHELL_CHANNEL_BINDING_MISMATCH: { category: "auth", retryable: false },
};

/**
 * Retry decision for one registered code (error-contract §3).
 *
 * - `retry`: the contract marks the code retryable. A retry is never an
 *   implicit success; `precondition` (when present) must be satisfied first
 *   (STATE_CONFLICT: re-read authoritative state, never reuse a stale
 *   `expected_state_version`).
 * - `reconcile`: retryable only through reconciliation-or-quarantine
 *   (EFFECT_OUTCOME_UNKNOWN); blind re-dispatch is forbidden.
 * - `non-retryable`: the contract forbids retry. `registered: false` marks
 *   an unregistered code, which is itself a defect (§2) and fails closed.
 */
export type RetryClassification =
  | { kind: "retry"; registered: true; precondition?: "reread-authoritative-state" }
  | { kind: "reconcile"; registered: true }
  | { kind: "non-retryable"; registered: boolean };

/**
 * Classify a code from the pinned registry. `wireRetryable` is the copy of
 * `retryable` carried on the wire error envelope; on disagreement the
 * narrower (risk-non-expanding) interpretation wins, so `false` from either
 * source makes the outcome non-retryable.
 */
export function classifyError(code: string, wireRetryable?: boolean): RetryClassification {
  const entry = REGISTERED_ERRORS[code];
  if (!entry) {
    return { kind: "non-retryable", registered: false };
  }
  if (!entry.retryable || wireRetryable === false) {
    return { kind: "non-retryable", registered: true };
  }
  if (code === "EFFECT_OUTCOME_UNKNOWN") {
    return { kind: "reconcile", registered: true };
  }
  if (code === "STATE_CONFLICT") {
    return { kind: "retry", registered: true, precondition: "reread-authoritative-state" };
  }
  return { kind: "retry", registered: true };
}

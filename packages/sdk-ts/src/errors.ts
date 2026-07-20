/**
 * Contract-driven retry classification over the GENERATED error registry
 * binding (docs/standards/error-contract.md §2/§3; REQ-ERR-001, REQ-ERR-002;
 * `@cognitiveos/contracts-ts` `errorRegistry`, codegen 0.2.0).
 *
 * The former hand-pinned 55-code table and its test-time YAML re-read are
 * gone: `errorRegistry.ERROR_REGISTRY` is generated from
 * `specs/registry/errors.yaml` and parity-tested against it inside
 * contracts-ts, so this module only adds the §3 retry semantics on top.
 */

import { errorRegistry } from "@cognitiveos/contracts-ts";

/** Re-exported registry surface consumed by this SDK. */
export type RegisteredErrorCode = errorRegistry.RegisteredErrorCode;
export type RegisteredError = errorRegistry.RegisteredError;
export const ERROR_REGISTRY = errorRegistry.ERROR_REGISTRY;
export const ERROR_REGISTRY_DIGEST: string = errorRegistry.REGISTRY_DIGEST;

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
 * Classify a code from the generated registry. `wireRetryable` is the copy
 * of `retryable` carried on the wire error envelope; on disagreement the
 * narrower (risk-non-expanding) interpretation wins, so `false` from either
 * source makes the outcome non-retryable.
 */
export function classifyError(code: string, wireRetryable?: boolean): RetryClassification {
  const registered = errorRegistry.parseErrorCode(code);
  if (registered === undefined) {
    return { kind: "non-retryable", registered: false };
  }
  const entry = ERROR_REGISTRY[registered];
  if (!entry.retryable || wireRetryable === false) {
    return { kind: "non-retryable", registered: true };
  }
  if (registered === "EFFECT_OUTCOME_UNKNOWN") {
    return { kind: "reconcile", registered: true };
  }
  if (registered === "STATE_CONFLICT") {
    return { kind: "retry", registered: true, precondition: "reread-authoritative-state" };
  }
  return { kind: "retry", registered: true };
}

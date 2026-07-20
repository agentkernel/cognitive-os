/**
 * Retry-classification tests over the generated error registry
 * (docs/standards/error-contract.md §3; REQ-ERR-001/002).
 *
 * The former test-time YAML re-read is gone: registry↔errors.yaml parity is
 * proven inside contracts-ts (codegen 0.2.0 parity tests). Here we pin the
 * §3 semantics layered on top of the generated table.
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import { classifyError, ERROR_REGISTRY, ERROR_REGISTRY_DIGEST } from "./errors.js";

test("classification consumes the generated registry (55 codes, digest constant)", () => {
  assert.equal(Object.keys(ERROR_REGISTRY).length, 55);
  assert.match(ERROR_REGISTRY_DIGEST, /^sha256:[0-9a-f]{64}$/);
});

test("every registered code classifies consistently with its generated retryable flag", () => {
  for (const entry of Object.values(ERROR_REGISTRY)) {
    const classification = classifyError(entry.code);
    if (!entry.retryable) {
      assert.deepEqual(
        classification,
        { kind: "non-retryable", registered: true },
        `${entry.code} must be non-retryable`,
      );
    } else {
      assert.notEqual(
        classification.kind,
        "non-retryable",
        `${entry.code} is contract-retryable and must not classify as non-retryable`,
      );
      assert.ok(classification.registered, `${entry.code} must classify as registered`);
    }
  }
});

test("classification is registry-driven: retryable false is never retried", () => {
  for (const code of ["CONTEXT_AUTH_DENIED", "EFFECT_IDEMPOTENCY_CONFLICT", "CANCEL_TOO_LATE"]) {
    assert.deepEqual(classifyError(code), { kind: "non-retryable", registered: true });
  }
});

test("plain retryable codes classify as retry without preconditions", () => {
  assert.deepEqual(classifyError("STATE_STORE_UNAVAILABLE"), { kind: "retry", registered: true });
  assert.deepEqual(classifyError("WATCH_CURSOR_STALE"), { kind: "retry", registered: true });
});

test("EFFECT_OUTCOME_UNKNOWN retries only through reconciliation, never blind re-dispatch", () => {
  assert.deepEqual(classifyError("EFFECT_OUTCOME_UNKNOWN"), {
    kind: "reconcile",
    registered: true,
  });
});

test("STATE_CONFLICT retry requires re-reading authoritative state", () => {
  assert.deepEqual(classifyError("STATE_CONFLICT"), {
    kind: "retry",
    registered: true,
    precondition: "reread-authoritative-state",
  });
});

test("an unregistered code is a defect and fails closed as non-retryable", () => {
  assert.deepEqual(classifyError("TOTALLY_MADE_UP"), { kind: "non-retryable", registered: false });
});

test("a wire envelope claiming retryable for a non-retryable registered code stays non-retryable", () => {
  // Registry truth wins with the narrower (risk-non-expanding) interpretation.
  assert.equal(classifyError("CONTEXT_AUTH_DENIED", true).kind, "non-retryable");
  // And a wire retryable=false narrows a registry-retryable code to non-retryable.
  assert.equal(classifyError("STATE_STORE_UNAVAILABLE", false).kind, "non-retryable");
});

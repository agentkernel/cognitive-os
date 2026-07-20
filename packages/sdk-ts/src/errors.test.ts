/**
 * Registered-error-code table and retry classification tests.
 *
 * Drift gate: `REGISTERED_ERRORS` must match `specs/registry/errors.yaml`
 * exactly (code set, category, retryable). The table is a pinned copy only
 * because contracts codegen does not yet emit an errors.yaml binding
 * (registered as a Lane-CTR contract gap in the 20260720 lane-tsc handoff);
 * this test turns any registry drift into a red build.
 *
 * Classification semantics under test: docs/standards/error-contract.md §3
 * — retryability is contract, not heuristic; EFFECT_OUTCOME_UNKNOWN retries
 * only through reconciliation; STATE_CONFLICT retries only after re-reading
 * authoritative state.
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { classifyError, REGISTERED_ERRORS } from "./errors.js";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const ERRORS_YAML = path.join(REPO_ROOT, "specs", "registry", "errors.yaml");

interface RegistryEntry {
  code: string;
  category: string;
  retryable: boolean;
}

interface PartialEntry {
  code: string;
  category?: string | undefined;
  retryable?: boolean | undefined;
}

/**
 * Minimal line-based reader for the fixed errors.yaml layout (list items
 * `- code:` with 2-space-indented `category:` / `retryable:` scalars).
 * Test-only: the runtime table must not depend on repo files.
 */
function readErrorRegistry(): RegistryEntry[] {
  const entries: RegistryEntry[] = [];
  let current: PartialEntry | undefined;
  for (const line of readFileSync(ERRORS_YAML, "utf-8").split(/\r?\n/)) {
    const codeMatch = /^-\s+code:\s*(\S+)\s*$/.exec(line);
    const code = codeMatch?.[1];
    if (code !== undefined) {
      if (current) {
        entries.push(assertComplete(current));
      }
      current = { code };
      continue;
    }
    const fieldMatch = /^\s+(category|retryable):\s*(\S+)\s*$/.exec(line);
    const fieldValue = fieldMatch?.[2];
    if (fieldMatch && fieldValue !== undefined && current) {
      if (fieldMatch[1] === "category") {
        current.category = fieldValue;
      } else {
        assert.ok(fieldValue === "true" || fieldValue === "false", `bad retryable: ${line}`);
        current.retryable = fieldValue === "true";
      }
    }
  }
  if (current) {
    entries.push(assertComplete(current));
  }
  return entries;
}

function assertComplete(entry: PartialEntry): RegistryEntry {
  const { code, category, retryable } = entry;
  assert.ok(category, `registry entry ${code} without category`);
  assert.ok(typeof retryable === "boolean", `registry entry ${code} without retryable`);
  return { code, category, retryable };
}

test("REGISTERED_ERRORS matches specs/registry/errors.yaml exactly (55 codes)", () => {
  const registry = readErrorRegistry();
  assert.equal(registry.length, 55, "registered code count changed; regenerate the pinned table");
  assert.equal(Object.keys(REGISTERED_ERRORS).length, registry.length);
  for (const entry of registry) {
    const pinned = REGISTERED_ERRORS[entry.code];
    assert.ok(pinned, `missing pinned entry for ${entry.code}`);
    assert.equal(pinned.category, entry.category, `${entry.code} category drifted`);
    assert.equal(pinned.retryable, entry.retryable, `${entry.code} retryable drifted`);
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

/**
 * Generated-binding integration with the canonical encoding layer, TS twin
 * of `crates/cognitive-contracts/tests/generated_types.rs` (ADR-0006
 * acceptance: generated types compile under strict settings and describe
 * real contract instances; canonical digests stay byte-identical).
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { canonicalize, digest } from "./canonical.js";
import type { Effect, EffectState } from "./generated/effect.js";
import {
  ERROR_REGISTRY,
  REGISTERED_ERRORS,
  REGISTRY_DIGEST,
  parseErrorCode,
  type RegisteredErrorCode,
} from "./generated/error-registry.js";
import { SCHEMA_DIGESTS } from "./generated/index.js";
import type { IntentInterpretation } from "./generated/intent-interpretation.js";
import type { ManagementActionProposal } from "./generated/management-action-proposal.js";
import type { PrivilegedManagementSession } from "./generated/privileged-management-session.js";
import type { StrongReference } from "./generated/object-reference.js";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

function positiveEffect(): Effect {
  const vector = JSON.parse(
    readFileSync(
      path.join(REPO_ROOT, "conformance", "vectors", "governed-object-legacy-strongref-001.json"),
      "utf-8",
    ),
  ) as { input: { object: Record<string, unknown> } };
  const intentRef: StrongReference = {
    kind: "strong",
    id: "01890a5d-ac96-774b-bcce-b302099a805d",
    object_version: 1,
    content_digest: "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
  };
  // The vector object minus its legacy reference is exactly a valid Effect.
  return { ...vector.input.object, intent_ref: intentRef } as unknown as Effect;
}

test("generated Effect type describes the migrated instance; digest preserved", () => {
  const effect = positiveEffect();
  const state: EffectState = effect.state;
  assert.equal(state, "PROPOSED");
  assert.equal(effect.header.schema_version, "cognitiveos.effect/0.2");
  assert.equal(effect.intent_ref.kind, "strong");

  const domain = "governed-object-content/0.1";
  const original = digest(canonicalize(JSON.stringify(effect)), domain);
  const roundTripped = digest(
    canonicalize(JSON.stringify(JSON.parse(JSON.stringify(effect)))),
    domain,
  );
  assert.equal(original, roundTripped, "canonical digest must survive the typed round trip");
});

test("generated literal unions reject wrong states at compile time", () => {
  const effect = positiveEffect();
  // @ts-expect-error - "NOT_A_STATE" is not a member of EffectState.
  const bad: EffectState = "NOT_A_STATE";
  void bad;
  // @ts-expect-error - legacy strongRef members are not part of StrongReference.
  const legacy: StrongReference = { id: "x", version: 4, digest: "sha256:0" };
  void legacy;
  assert.ok(effect.verification.status);
});

test("SCHEMA_DIGESTS constants match the live schema files (35 generated modules)", () => {
  // Gap 5 of the 20260720 lane-tsc handoff: the digest is a RUNTIME
  // constant clients pin envelope `schema_digest` with; it must equal the
  // re-derived canonical digest of the live schema (the schema-bundle
  // manifest per-asset recipe).
  const entries = Object.entries(SCHEMA_DIGESTS);
  assert.equal(entries.length, 35, "generated schema module count drifted");
  for (const [file, pinned] of entries) {
    const raw = readFileSync(path.join(REPO_ROOT, "specs", "schemas", file), "utf-8");
    const live = digest(canonicalize(raw), "schema-bundle/0.1");
    assert.equal(pinned, live, `${file}: SCHEMA_DIGESTS entry is stale`);
  }
});
test("M5 consumer bindings export required members and digest pins", () => {
  // @ts-expect-error - registered required members cannot be omitted.
  const interpretation: IntentInterpretation = {};
  // @ts-expect-error - registered required members cannot be omitted.
  const session: PrivilegedManagementSession = {};
  // @ts-expect-error - registered required members cannot be omitted.
  const proposal: ManagementActionProposal = {};
  void interpretation;
  void session;
  void proposal;
  assert.equal(Object.hasOwn(SCHEMA_DIGESTS, "intent-interpretation.schema.json"), true);
  assert.equal(Object.hasOwn(SCHEMA_DIGESTS, "privileged-management-session.schema.json"), true);
  assert.equal(Object.hasOwn(SCHEMA_DIGESTS, "management-action-proposal.schema.json"), true);
});

test("generated error registry is table-complete and fail-closed on unknown codes", () => {
  // Gap 2 of the 20260720 lane-tsc handoff. The entry-by-entry parity with
  // specs/registry/errors.yaml is pinned by the Rust twin
  // (`error_registry_matches_errors_yaml`) plus the CI regenerate-and-diff
  // gate; this side checks the table invariants and the lookup surface.
  assert.equal(REGISTERED_ERRORS.length, 55, "registered code count drifted");
  assert.match(REGISTRY_DIGEST, /^sha256:[0-9a-f]{64}$/);
  const codes = new Set(REGISTERED_ERRORS.map((entry) => entry.code));
  assert.equal(codes.size, REGISTERED_ERRORS.length, "duplicate code in the table");
  for (const entry of REGISTERED_ERRORS) {
    assert.equal(ERROR_REGISTRY[entry.code], entry, `${entry.code}: lookup diverged`);
    assert.equal(parseErrorCode(entry.code), entry.code);
    assert.ok(entry.description.length > 0, `${entry.code}: empty description`);
  }
  // Contract-driven retry classification stays registry truth
  // (docs/standards/error-contract.md section 3).
  assert.equal(ERROR_REGISTRY["STATE_CONFLICT"].retryable, true);
  assert.equal(ERROR_REGISTRY["EFFECT_OUTCOME_UNKNOWN"].retryable, true);
  assert.equal(ERROR_REGISTRY["EFFECT_IDEMPOTENCY_CONFLICT"].retryable, false);
  assert.equal(ERROR_REGISTRY["CONTEXT_AUTH_DENIED"].retryable, false);
  assert.equal(parseErrorCode("NOT_A_REGISTERED_CODE"), undefined);
  // Fail-closed lookups never fall through to Object.prototype members.
  assert.equal(parseErrorCode("toString"), undefined);
  // @ts-expect-error - a non-registered literal is not a RegisteredErrorCode.
  const bad: RegisteredErrorCode = "NOT_A_REGISTERED_CODE";
  void bad;
});

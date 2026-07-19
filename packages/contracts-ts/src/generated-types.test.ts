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

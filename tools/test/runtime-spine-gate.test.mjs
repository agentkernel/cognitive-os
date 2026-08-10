import assert from "node:assert/strict";
import test from "node:test";

import {
  RUNTIME_SPINE_REQUIRED_OBSERVATIONS,
  buildRuntimeSpineGateSuiteReport,
} from "../src/runtime-spine-gate.mjs";

const digest = (character) => `sha256:${character.repeat(64)}`;

function campaign(overrides = {}) {
  const observations = Object.fromEntries(
    RUNTIME_SPINE_REQUIRED_OBSERVATIONS.map((name) => [name, true]),
  );
  return {
    campaign_id: "runtime-spine-gates/1",
    claim_scope: "non-claim",
    target_gates: ["B02", "B04", "B05", "B12"],
    suite_digest: digest("a"),
    trace_digest: digest("b"),
    default_path_confirmation_count: 1,
    observations,
    ...overrides,
  };
}

test("buildRuntimeSpineGateSuiteReport records complete non-claim observations", () => {
  const first = buildRuntimeSpineGateSuiteReport(campaign());
  const second = buildRuntimeSpineGateSuiteReport(campaign());

  assert.equal(first.report.claim_scope, "non-claim");
  assert.deepEqual(first.report.target_gates, ["B02", "B04", "B05", "B12"]);
  assert.equal(first.report.default_path_confirmation_count, 1);
  assert.equal(first.report.observations.length, RUNTIME_SPINE_REQUIRED_OBSERVATIONS.length);
  assert.equal(first.report_digest, second.report_digest);
  assert.ok(first.report.non_claims.includes("does not set Gate state"));
});

test("buildRuntimeSpineGateSuiteReport rejects incomplete observations and authority claims", () => {
  const incomplete = campaign();
  incomplete.observations.no_false_completion = false;
  assert.throws(
    () => buildRuntimeSpineGateSuiteReport(incomplete),
    /no_false_completion/,
  );

  const claimed = campaign({ gate: "pass" });
  assert.throws(() => buildRuntimeSpineGateSuiteReport(claimed), /forbidden/);

  const wrongGates = campaign({ target_gates: ["B02", "B04"] });
  assert.throws(() => buildRuntimeSpineGateSuiteReport(wrongGates), /target_gates/);

  const tooManyConfirmations = campaign({ default_path_confirmation_count: 2 });
  assert.throws(
    () => buildRuntimeSpineGateSuiteReport(tooManyConfirmations),
    /default_path_confirmation_count/,
  );

  const missingAdrCheck = campaign();
  missingAdrCheck.observations.adr0018_local_native_exception_absent_or_replaced = false;
  assert.throws(
    () => buildRuntimeSpineGateSuiteReport(missingAdrCheck),
    /adr0018_local_native_exception_absent_or_replaced/,
  );
});

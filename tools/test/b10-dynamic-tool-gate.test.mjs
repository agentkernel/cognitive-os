import assert from "node:assert/strict";
import test from "node:test";

import {
  B10_REQUIRED_OBSERVATIONS,
  buildB10DynamicToolGateSuiteReport,
} from "../src/b10-dynamic-tool-gate.mjs";

const digest = (character) => `sha256:${character.repeat(64)}`;

function campaign(overrides = {}) {
  const observations = Object.fromEntries(
    B10_REQUIRED_OBSERVATIONS.map((name) => [name, true]),
  );
  return {
    campaign_id: "B10-dynamic-tool-ecosystem/1",
    claim_scope: "non-claim",
    target_gates: ["B10"],
    suite_digest: digest("a"),
    trace_digest: digest("b"),
    observations,
    ...overrides,
  };
}

test("buildB10DynamicToolGateSuiteReport records complete non-claim observations", () => {
  const first = buildB10DynamicToolGateSuiteReport(campaign());
  const second = buildB10DynamicToolGateSuiteReport(campaign());

  assert.equal(first.report.claim_scope, "non-claim");
  assert.deepEqual(first.report.target_gates, ["B10"]);
  assert.equal(first.report.observations.length, B10_REQUIRED_OBSERVATIONS.length);
  assert.equal(first.report_digest, second.report_digest);
  assert.ok(first.report.non_claims.includes("does not set Gate state"));
});

test("buildB10DynamicToolGateSuiteReport rejects incomplete observations and authority claims", () => {
  const incomplete = campaign();
  incomplete.observations.sandbox_bypass_rejected = false;
  assert.throws(
    () => buildB10DynamicToolGateSuiteReport(incomplete),
    /sandbox_bypass_rejected/,
  );

  const claimed = campaign({ gate: "pass" });
  assert.throws(() => buildB10DynamicToolGateSuiteReport(claimed), /forbidden/);

  const wrongGates = campaign({ target_gates: ["B10", "B09"] });
  assert.throws(() => buildB10DynamicToolGateSuiteReport(wrongGates), /target_gates/);

  const wrongId = campaign({ campaign_id: "other" });
  assert.throws(() => buildB10DynamicToolGateSuiteReport(wrongId), /campaign_id/);
});

import assert from "node:assert/strict";
import test from "node:test";

import {
  B09_REQUIRED_OBSERVATIONS,
  buildB09ManagedPiGateSuiteReport,
} from "../src/b09-managed-pi-gate.mjs";

const digest = (character) => `sha256:${character.repeat(64)}`;

function campaign(overrides = {}) {
  const observations = Object.fromEntries(
    B09_REQUIRED_OBSERVATIONS.map((name) => [name, true]),
  );
  return {
    campaign_id: "B09-managed-pi-sidecar/1",
    claim_scope: "non-claim",
    target_gates: ["B09"],
    suite_digest: digest("a"),
    trace_digest: digest("b"),
    observations,
    ...overrides,
  };
}

test("buildB09ManagedPiGateSuiteReport records complete non-claim observations", () => {
  const first = buildB09ManagedPiGateSuiteReport(campaign());
  const second = buildB09ManagedPiGateSuiteReport(campaign());

  assert.equal(first.report.claim_scope, "non-claim");
  assert.deepEqual(first.report.target_gates, ["B09"]);
  assert.equal(first.report.observations.length, B09_REQUIRED_OBSERVATIONS.length);
  assert.equal(first.report_digest, second.report_digest);
  assert.ok(first.report.non_claims.includes("does not set Gate state"));
});

test("buildB09ManagedPiGateSuiteReport rejects incomplete observations and authority claims", () => {
  const incomplete = campaign();
  incomplete.observations.orphan_no_reattach = false;
  assert.throws(() => buildB09ManagedPiGateSuiteReport(incomplete), /orphan_no_reattach/);

  const claimed = campaign({ gate: "pass" });
  assert.throws(() => buildB09ManagedPiGateSuiteReport(claimed), /forbidden/);

  const wrongGates = campaign({ target_gates: ["B09", "B08"] });
  assert.throws(() => buildB09ManagedPiGateSuiteReport(wrongGates), /target_gates/);

  const wrongId = campaign({ campaign_id: "other" });
  assert.throws(() => buildB09ManagedPiGateSuiteReport(wrongId), /campaign_id/);
});

import assert from "node:assert/strict";
import test from "node:test";

import {
  GMVP_REQUIRED_OBSERVATIONS,
  buildGmvpLinuxGateSuiteReport,
} from "../src/gmvp-linux-gate.mjs";

const digest = (character) => `sha256:${character.repeat(64)}`;

function campaign(overrides = {}) {
  const observations = Object.fromEntries(
    GMVP_REQUIRED_OBSERVATIONS.map((name) => [name, true]),
  );
  return {
    campaign_id: "GMVP-LINUX-composition/1",
    claim_scope: "non-claim",
    target_gates: ["GMVP-LINUX"],
    suite_digest: digest("c"),
    trace_digest: digest("d"),
    observations,
    ...overrides,
  };
}

test("buildGmvpLinuxGateSuiteReport records complete non-claim composition", () => {
  const first = buildGmvpLinuxGateSuiteReport(campaign());
  const second = buildGmvpLinuxGateSuiteReport(campaign());

  assert.equal(first.report.claim_scope, "non-claim");
  assert.deepEqual(first.report.target_gates, ["GMVP-LINUX"]);
  assert.equal(first.report.observations.length, GMVP_REQUIRED_OBSERVATIONS.length);
  assert.equal(first.report_digest, second.report_digest);
  assert.ok(first.report.non_claims.includes("does not set Gate state"));
});

test("buildGmvpLinuxGateSuiteReport rejects incomplete composition and authority claims", () => {
  const incomplete = campaign();
  incomplete.observations.b08_mvp_pass = false;
  assert.throws(() => buildGmvpLinuxGateSuiteReport(incomplete), /b08_mvp_pass/);

  const claimed = campaign({ release: "1.0.0" });
  assert.throws(() => buildGmvpLinuxGateSuiteReport(claimed), /forbidden/);

  const wrongGates = campaign({ target_gates: ["GMVP-LINUX", "B08"] });
  assert.throws(() => buildGmvpLinuxGateSuiteReport(wrongGates), /target_gates/);

  const wrongId = campaign({ campaign_id: "other" });
  assert.throws(() => buildGmvpLinuxGateSuiteReport(wrongId), /campaign_id/);
});

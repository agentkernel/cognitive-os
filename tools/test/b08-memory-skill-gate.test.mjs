import assert from "node:assert/strict";
import test from "node:test";

import {
  B08_REQUIRED_OBSERVATIONS,
  buildB08MemorySkillGateSuiteReport,
} from "../src/b08-memory-skill-gate.mjs";

const digest = (character) => `sha256:${character.repeat(64)}`;

function campaign(overrides = {}) {
  const observations = Object.fromEntries(
    B08_REQUIRED_OBSERVATIONS.map((name) => [name, true]),
  );
  return {
    campaign_id: "B08-memory-skill-consumption/1",
    claim_scope: "non-claim",
    target_gates: ["B08"],
    suite_digest: digest("a"),
    trace_digest: digest("b"),
    observations,
    ...overrides,
  };
}

test("buildB08MemorySkillGateSuiteReport records complete non-claim observations", () => {
  const first = buildB08MemorySkillGateSuiteReport(campaign());
  const second = buildB08MemorySkillGateSuiteReport(campaign());

  assert.equal(first.report.claim_scope, "non-claim");
  assert.deepEqual(first.report.target_gates, ["B08"]);
  assert.equal(first.report.observations.length, B08_REQUIRED_OBSERVATIONS.length);
  assert.equal(first.report_digest, second.report_digest);
  assert.ok(first.report.non_claims.includes("does not set Gate state"));
});

test("buildB08MemorySkillGateSuiteReport rejects incomplete observations and authority claims", () => {
  const incomplete = campaign();
  incomplete.observations.task_consumption_channel_isolation = false;
  assert.throws(
    () => buildB08MemorySkillGateSuiteReport(incomplete),
    /task_consumption_channel_isolation/,
  );

  const claimed = campaign({ gate: "pass" });
  assert.throws(() => buildB08MemorySkillGateSuiteReport(claimed), /forbidden/);

  const wrongGates = campaign({ target_gates: ["B08", "B09"] });
  assert.throws(() => buildB08MemorySkillGateSuiteReport(wrongGates), /target_gates/);

  const wrongId = campaign({ campaign_id: "other" });
  assert.throws(() => buildB08MemorySkillGateSuiteReport(wrongId), /campaign_id/);
});

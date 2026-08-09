import assert from "node:assert/strict";
import test from "node:test";

import { buildUcrRunReport } from "../src/ucr-runner.mjs";

const digest = (character) => `sha256:${character.repeat(64)}`;

function measurements(tokenCount) {
  return {
    stable: { repeated_input_tokens: tokenCount },
    changed: { repeated_input_tokens: tokenCount + 10 },
    full_replay: { repeated_input_tokens: tokenCount + 20 },
    tool_calls: 1,
    tool_failures: 0,
  };
}

function rawRun() {
  return {
    scenario_id: "UCR-01",
    claim_scope: "non-claim",
    resource_families: ["runtime", "task", "context", "tool", "skill", "memory"],
    fixture_digest: digest("a"),
    trace_digest: digest("b"),
    baseline_digest: digest("c"),
    measurements: measurements(80),
  };
}

function stableBaseline() {
  return { baseline_digest: digest("c"), measurements: measurements(100) };
}

test("buildUcrRunReport emits a digest-bound non-claim report", () => {
  const firstResult = buildUcrRunReport(rawRun(), stableBaseline());
  const secondResult = buildUcrRunReport(rawRun(), stableBaseline());

  assert.equal(firstResult.report.claim_scope, "non-claim");
  assert.deepEqual(firstResult.report.resource_families, [
    "memory",
    "skill",
    "tool",
    "context",
    "task",
    "runtime",
  ]);
  assert.equal(firstResult.report.observations.stable_repeated_input_delta, -20);
  assert.equal(firstResult.report_digest, secondResult.report_digest);
});

test("buildUcrRunReport rejects authority claims and unpinned baselines", () => {
  const claimedRun = rawRun();
  claimedRun.gate = "pass";
  assert.throws(() => buildUcrRunReport(claimedRun, stableBaseline()), /forbidden/);

  const mismatchedBaseline = stableBaseline();
  mismatchedBaseline.baseline_digest = digest("d");
  assert.throws(
    () => buildUcrRunReport(rawRun(), mismatchedBaseline),
    /does not match the pinned stable baseline/,
  );
});

test("buildUcrRunReport rejects incomplete fixtures and unbounded measurements", () => {
  const incompleteRun = rawRun();
  incompleteRun.resource_families.pop();
  assert.throws(() => buildUcrRunReport(incompleteRun, stableBaseline()), /cover each/);

  const invalidMeasurementRun = rawRun();
  invalidMeasurementRun.measurements.tool_failures = -1;
  assert.throws(
    () => buildUcrRunReport(invalidMeasurementRun, stableBaseline()),
    /tool_failures/,
  );
});

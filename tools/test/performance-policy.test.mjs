import assert from "node:assert/strict";
import { test } from "node:test";
import {
  buildGovernanceAbCampaignReport,
  evaluateModuleRegressionFloor,
  runDeterministicModuleBenchmark,
  summarizeDurationSamples,
  validatePerformanceReportPolicy,
} from "../src/performance-policy.mjs";

function createValidNonInferiorityReport() {
  return {
    benchmark_manifest: {
      samples: 20,
    },
    slo_profile: {
      thresholds: [
        {
          metric: "governed_latency_ms",
          release_gate: true,
          on_breach: "block_release",
        },
      ],
    },
    metrics: [
      {
        name: "governed_latency_ms",
        p50: 2,
        p95: 4,
        p99: 7,
        sample_count: 20,
        confidence_interval: {
          low: 1,
          high: 8,
        },
      },
    ],
    comparison: {
      arms: [
        {
          arm_id: "A",
          arm_kind: "native_baseline",
        },
        {
          arm_id: "B",
          arm_kind: "governance_only",
        },
      ],
      claim_level: "non_inferiority",
      preregistration_ref: "docs/checkpoints/p7-t04-preregistration.md",
      results: [
        {
          arm_a: "A",
          arm_b: "B",
          confidence_interval: {
            low: -1,
            high: 2,
          },
        },
      ],
    },
  };
}

test("duration summaries use deterministic nearest-rank percentiles", () => {
  const summary = summarizeDurationSamples([10, 1, 5, 3, 20]);

  assert.deepEqual(summary, {
    sample_count: 5,
    p50: 5,
    p95: 20,
    p99: 20,
    minimum: 1,
    maximum: 20,
  });
});

test("module benchmark binds fixture and emits digest-bound raw evidence", () => {
  let observedOperationCount = 0;
  const benchmarkEvidence = runDeterministicModuleBenchmark({
    benchmarkId: "canonical-report-serialization",
    fixtureDigest: `sha256:${"a".repeat(64)}`,
    iterations: 4,
    warmupIterations: 2,
    operation: () => {
      observedOperationCount += 1;
      JSON.stringify({ authority: "daemon", count: observedOperationCount });
    },
  });

  assert.equal(observedOperationCount, 6);
  assert.equal(benchmarkEvidence.sample_count, 4);
  assert.equal(benchmarkEvidence.raw_samples.length, 4);
  assert.match(benchmarkEvidence.evidence_digest, /^sha256:[a-f0-9]{64}$/);
  assert.ok(benchmarkEvidence.p50 <= benchmarkEvidence.p95);
  assert.ok(benchmarkEvidence.p95 <= benchmarkEvidence.p99);
});

test("valid A/B non-inferiority report passes semantic policy", () => {
  assert.deepEqual(validatePerformanceReportPolicy(createValidNonInferiorityReport()), []);
});

test("semantic policy rejects invalid percentiles and release threshold action", () => {
  const report = createValidNonInferiorityReport();
  report.metrics[0].p50 = 9;
  report.slo_profile.thresholds[0].on_breach = "alert_only";

  const errorCodes = validatePerformanceReportPolicy(report).map((error) => error.code);
  assert.ok(errorCodes.includes("PERFORMANCE_PERCENTILE_ORDER_INVALID"));
  assert.ok(errorCodes.includes("PERFORMANCE_RELEASE_THRESHOLD_BREACH_ACTION_INVALID"));
});

test("builder samples cannot be promoted to measured non-inferiority", () => {
  const report = createValidNonInferiorityReport();
  report.benchmark_manifest.execution_note = "sample_or_builder_only";

  const errorCodes = validatePerformanceReportPolicy(report).map((error) => error.code);
  assert.ok(errorCodes.includes("PERFORMANCE_BUILDER_SAMPLE_CANNOT_CLAIM"));
});

test("significant benefit requires four arms, two workloads, and ablation", () => {
  const report = createValidNonInferiorityReport();
  report.comparison.claim_level = "significant_benefit";
  report.comparison.workload_family = ["W1"];

  const errorCodes = validatePerformanceReportPolicy(report).map((error) => error.code);
  assert.ok(errorCodes.includes("PERFORMANCE_BENEFIT_ARM_MISSING"));
  assert.ok(errorCodes.includes("PERFORMANCE_BENEFIT_WORKLOAD_COVERAGE_INCOMPLETE"));
  assert.ok(errorCodes.includes("PERFORMANCE_BENEFIT_ABLATION_MISSING"));
});

test("benchmark input validation rejects missing fixture provenance", () => {
  assert.throws(
    () =>
      runDeterministicModuleBenchmark({
        benchmarkId: "context-filter",
        fixtureDigest: "floating-fixture",
        iterations: 1,
        operation: () => undefined,
      }),
    /fixtureDigest must be a sha256 digest/,
  );
});

test("module regression floors reject floating-CI release gates and record breaches", () => {
  assert.throws(
    () =>
      evaluateModuleRegressionFloor({
        environmentKind: "floating-ci",
        observations: [{ benchmark_id: "context-cache-full-key-hit", p95: 10 }],
        floors: [
          {
            benchmark_id: "context-cache-full-key-hit",
            p95_ceiling_nanoseconds: 5,
            release_gate: true,
            on_breach: "block_release",
          },
        ],
      }),
    /floating CI cannot evaluate release-gating/,
  );

  const evaluation = evaluateModuleRegressionFloor({
    environmentKind: "floating-ci",
    observations: [{ benchmark_id: "context-cache-full-key-hit", p95: 40 }],
    floors: [
      {
        benchmark_id: "context-cache-full-key-hit",
        p95_ceiling_nanoseconds: 10,
        release_gate: false,
        on_breach: "record_only",
      },
    ],
  });
  assert.equal(evaluation.claim_level, "hypothesis");
  assert.equal(evaluation.release_hardware_evidence, false);
  assert.equal(evaluation.breaches[0].code, "PERFORMANCE_REGRESSION_FLOOR_BREACHED");
});

test("governance A/B campaign requires preregistration and fixed-native evidence", () => {
  const campaign = {
    claim_level: "non_inferiority",
    preregistration_ref: "docs/checkpoints/20260810-personal-p7-t04-ab-preregistration.md",
    source_revision: "a".repeat(40),
    environment_kind: "fixed-native",
    environment_digest: `sha256:${"b".repeat(64)}`,
    denominator: { started_attempts: 6, retained_attempts: 6 },
    safety: { critical_safety_failures: 0, false_completions: 0 },
    metrics: {
      governed_latency_ms: {
        p50: 2,
        p95: 4,
        p99: 7,
        confidence_interval: { low: 1, high: 8 },
      },
    },
    comparison_confidence_interval: { low: -1, high: 2 },
  };

  const first = buildGovernanceAbCampaignReport(campaign);
  const second = buildGovernanceAbCampaignReport(campaign);
  assert.equal(first.report.comparison.claim_level, "non_inferiority");
  assert.equal(first.report_digest, second.report_digest);
  assert.ok(first.report.non_claims.includes("does not block or pass GMVP-LINUX"));

  assert.throws(
    () =>
      buildGovernanceAbCampaignReport({
        ...campaign,
        environment_kind: "floating-ci",
      }),
    /fixed-native/,
  );
  assert.throws(
    () =>
      buildGovernanceAbCampaignReport({
        ...campaign,
        preregistration_ref: "",
      }),
    /preregistration_ref/,
  );
});

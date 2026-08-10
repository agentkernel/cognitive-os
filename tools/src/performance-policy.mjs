import { createHash } from "node:crypto";

const VALID_CLAIM_LEVELS = new Set([
  "hypothesis",
  "non_inferiority",
  "significant_benefit",
]);

function addPolicyError(errors, code, path, message) {
  errors.push({ code, path, message });
}

function isFiniteNumber(value) {
  return typeof value === "number" && Number.isFinite(value);
}

function validateConfidenceInterval(confidenceInterval, path, errors) {
  if (!confidenceInterval || typeof confidenceInterval !== "object") {
    addPolicyError(
      errors,
      "PERFORMANCE_CONFIDENCE_INTERVAL_MISSING",
      path,
      "a confidence interval is required",
    );
    return;
  }

  const lowerBound = confidenceInterval.low;
  const upperBound = confidenceInterval.high;
  if (!isFiniteNumber(lowerBound) || !isFiniteNumber(upperBound)) {
    addPolicyError(
      errors,
      "PERFORMANCE_CONFIDENCE_INTERVAL_NON_FINITE",
      path,
      "confidence interval bounds must be finite numbers",
    );
  } else if (lowerBound > upperBound) {
    addPolicyError(
      errors,
      "PERFORMANCE_CONFIDENCE_INTERVAL_REVERSED",
      path,
      "confidence interval low must not exceed high",
    );
  }
}

function validateMetric(metric, metricIndex, manifestSampleCount, errors) {
  const metricPath = `metrics[${metricIndex}]`;
  const percentileValues = [metric?.p50, metric?.p95, metric?.p99];
  if (!percentileValues.every(isFiniteNumber)) {
    addPolicyError(
      errors,
      "PERFORMANCE_PERCENTILE_NON_FINITE",
      metricPath,
      "p50, p95, and p99 must be finite numbers",
    );
  } else if (!(metric.p50 <= metric.p95 && metric.p95 <= metric.p99)) {
    addPolicyError(
      errors,
      "PERFORMANCE_PERCENTILE_ORDER_INVALID",
      metricPath,
      "percentiles must satisfy p50 <= p95 <= p99",
    );
  }

  if (!Number.isInteger(metric?.sample_count) || metric.sample_count < 1) {
    addPolicyError(
      errors,
      "PERFORMANCE_SAMPLE_COUNT_INVALID",
      `${metricPath}.sample_count`,
      "sample_count must be a positive integer",
    );
  } else if (
    Number.isInteger(manifestSampleCount) &&
    metric.sample_count > manifestSampleCount
  ) {
    addPolicyError(
      errors,
      "PERFORMANCE_SAMPLE_COUNT_EXCEEDS_MANIFEST",
      `${metricPath}.sample_count`,
      "metric sample_count must not exceed the benchmark manifest denominator",
    );
  }

  validateConfidenceInterval(
    metric?.confidence_interval,
    `${metricPath}.confidence_interval`,
    errors,
  );
}

function validateComparison(report, errors) {
  const comparison = report?.comparison;
  if (!comparison) {
    return;
  }

  if (!VALID_CLAIM_LEVELS.has(comparison.claim_level)) {
    addPolicyError(
      errors,
      "PERFORMANCE_CLAIM_LEVEL_INVALID",
      "comparison.claim_level",
      "claim_level must be hypothesis, non_inferiority, or significant_benefit",
    );
    return;
  }

  const arms = Array.isArray(comparison.arms) ? comparison.arms : [];
  const armIdentifiers = arms.map((arm) => arm?.arm_id).filter(Boolean);
  const uniqueArmIdentifiers = new Set(armIdentifiers);
  if (uniqueArmIdentifiers.size !== armIdentifiers.length) {
    addPolicyError(
      errors,
      "PERFORMANCE_COMPARISON_DUPLICATE_ARM",
      "comparison.arms",
      "comparison arm identifiers must be unique",
    );
  }

  const armKinds = new Set(arms.map((arm) => arm?.arm_kind));
  const requiresMeasuredComparison = comparison.claim_level !== "hypothesis";
  if (requiresMeasuredComparison) {
    for (const requiredArmKind of ["native_baseline", "governance_only"]) {
      if (!armKinds.has(requiredArmKind)) {
        addPolicyError(
          errors,
          "PERFORMANCE_NON_INFERIORITY_ARM_MISSING",
          "comparison.arms",
          `measured comparison requires ${requiredArmKind}`,
        );
      }
    }
    if (!Array.isArray(comparison.results) || comparison.results.length === 0) {
      addPolicyError(
        errors,
        "PERFORMANCE_COMPARISON_RESULTS_MISSING",
        "comparison.results",
        "measured comparison requires at least one result",
      );
    }
    if (!String(comparison.preregistration_ref ?? "").trim()) {
      addPolicyError(
        errors,
        "PERFORMANCE_PREREGISTRATION_MISSING",
        "comparison.preregistration_ref",
        "measured comparison requires a preregistration reference",
      );
    }
  }

  for (const [resultIndex, result] of (comparison.results ?? []).entries()) {
    const resultPath = `comparison.results[${resultIndex}]`;
    if (
      !uniqueArmIdentifiers.has(result?.arm_a) ||
      !uniqueArmIdentifiers.has(result?.arm_b)
    ) {
      addPolicyError(
        errors,
        "PERFORMANCE_COMPARISON_UNKNOWN_ARM",
        resultPath,
        "comparison result must reference two declared arms",
      );
    }
    validateConfidenceInterval(
      result?.confidence_interval,
      `${resultPath}.confidence_interval`,
      errors,
    );
  }

  if (comparison.claim_level === "significant_benefit") {
    for (const requiredArmKind of [
      "native_baseline",
      "governance_only",
      "optimized",
      "ablation",
    ]) {
      if (!armKinds.has(requiredArmKind)) {
        addPolicyError(
          errors,
          "PERFORMANCE_BENEFIT_ARM_MISSING",
          "comparison.arms",
          `significant benefit requires ${requiredArmKind}`,
        );
      }
    }
    const workloadFamilies = new Set(comparison.workload_family ?? []);
    if (workloadFamilies.size < 2) {
      addPolicyError(
        errors,
        "PERFORMANCE_BENEFIT_WORKLOAD_COVERAGE_INCOMPLETE",
        "comparison.workload_family",
        "significant benefit requires at least two workload families",
      );
    }
    if (
      !Array.isArray(comparison.ablation_results) ||
      comparison.ablation_results.length === 0
    ) {
      addPolicyError(
        errors,
        "PERFORMANCE_BENEFIT_ABLATION_MISSING",
        "comparison.ablation_results",
        "significant benefit requires ablation evidence",
      );
    }
  }
}

function validateReleaseThresholds(report, errors) {
  const metricNames = new Set((report?.metrics ?? []).map((metric) => metric?.name));
  for (const [thresholdIndex, threshold] of (
    report?.slo_profile?.thresholds ?? []
  ).entries()) {
    const thresholdPath = `slo_profile.thresholds[${thresholdIndex}]`;
    if (!metricNames.has(threshold?.metric)) {
      addPolicyError(
        errors,
        "PERFORMANCE_THRESHOLD_METRIC_MISSING",
        thresholdPath,
        "threshold must reference a metric present in the report",
      );
    }
    if (threshold?.release_gate === true && threshold?.on_breach !== "block_release") {
      addPolicyError(
        errors,
        "PERFORMANCE_RELEASE_THRESHOLD_BREACH_ACTION_INVALID",
        thresholdPath,
        "a release-gating threshold must block release on breach",
      );
    }
  }
}

export function validatePerformanceReportPolicy(report) {
  const errors = [];
  if (!report || typeof report !== "object" || Array.isArray(report)) {
    addPolicyError(
      errors,
      "PERFORMANCE_REPORT_INVALID",
      "$",
      "performance report must be an object",
    );
    return errors;
  }

  const serializedReport = JSON.stringify(report);
  const comparisonClaimLevel = report?.comparison?.claim_level;
  if (
    comparisonClaimLevel &&
    comparisonClaimLevel !== "hypothesis" &&
    /sample_or_builder_only|not_executed/i.test(serializedReport)
  ) {
    addPolicyError(
      errors,
      "PERFORMANCE_BUILDER_SAMPLE_CANNOT_CLAIM",
      "comparison.claim_level",
      "builder or unexecuted samples cannot support measured claims",
    );
  }

  const manifestSampleCount = report?.benchmark_manifest?.samples;
  const metrics = Array.isArray(report.metrics) ? report.metrics : [];
  const metricNames = metrics.map((metric) => metric?.name).filter(Boolean);
  if (new Set(metricNames).size !== metricNames.length) {
    addPolicyError(
      errors,
      "PERFORMANCE_METRIC_DUPLICATE",
      "metrics",
      "metric names must be unique within one report",
    );
  }
  for (const [metricIndex, metric] of metrics.entries()) {
    validateMetric(metric, metricIndex, manifestSampleCount, errors);
  }

  validateReleaseThresholds(report, errors);
  validateComparison(report, errors);
  return errors;
}

function percentileFromSortedSamples(sortedSamples, percentile) {
  const sampleIndex = Math.ceil(percentile * sortedSamples.length) - 1;
  return sortedSamples[Math.max(0, sampleIndex)];
}

export function summarizeDurationSamples(durationSamplesNanoseconds) {
  if (
    !Array.isArray(durationSamplesNanoseconds) ||
    durationSamplesNanoseconds.length === 0 ||
    durationSamplesNanoseconds.some(
      (durationNanoseconds) =>
        !isFiniteNumber(durationNanoseconds) || durationNanoseconds < 0,
    )
  ) {
    throw new TypeError("duration samples must be a non-empty array of finite non-negative numbers");
  }

  const sortedSamples = [...durationSamplesNanoseconds].sort(
    (leftDuration, rightDuration) => leftDuration - rightDuration,
  );
  return {
    sample_count: sortedSamples.length,
    p50: percentileFromSortedSamples(sortedSamples, 0.5),
    p95: percentileFromSortedSamples(sortedSamples, 0.95),
    p99: percentileFromSortedSamples(sortedSamples, 0.99),
    minimum: sortedSamples[0],
    maximum: sortedSamples.at(-1),
  };
}

export function runDeterministicModuleBenchmark({
  benchmarkId,
  fixtureDigest,
  iterations,
  operation,
  warmupIterations = 1,
}) {
  if (!String(benchmarkId ?? "").trim()) {
    throw new TypeError("benchmarkId is required");
  }
  if (!/^sha256:[a-f0-9]{64}$/.test(String(fixtureDigest ?? ""))) {
    throw new TypeError("fixtureDigest must be a sha256 digest");
  }
  if (!Number.isInteger(iterations) || iterations < 1) {
    throw new TypeError("iterations must be a positive integer");
  }
  if (!Number.isInteger(warmupIterations) || warmupIterations < 0) {
    throw new TypeError("warmupIterations must be a non-negative integer");
  }
  if (typeof operation !== "function") {
    throw new TypeError("operation must be a function");
  }

  for (let warmupIndex = 0; warmupIndex < warmupIterations; warmupIndex += 1) {
    operation(warmupIndex);
  }

  const durationSamplesNanoseconds = [];
  for (let iterationIndex = 0; iterationIndex < iterations; iterationIndex += 1) {
    const startTimeNanoseconds = process.hrtime.bigint();
    operation(iterationIndex);
    const endTimeNanoseconds = process.hrtime.bigint();
    durationSamplesNanoseconds.push(Number(endTimeNanoseconds - startTimeNanoseconds));
  }

  const summary = summarizeDurationSamples(durationSamplesNanoseconds);
  const evidencePayload = {
    benchmark_id: benchmarkId,
    fixture_digest: fixtureDigest,
    unit: "nanoseconds",
    warmup_iterations: warmupIterations,
    ...summary,
  };
  const evidenceDigest = createHash("sha256")
    .update(JSON.stringify(evidencePayload))
    .digest("hex");

  return {
    ...evidencePayload,
    evidence_digest: `sha256:${evidenceDigest}`,
    raw_samples: durationSamplesNanoseconds,
  };
}

/**
 * Evaluate module regression floors. Floating CI may track hypothesis floors
 * but cannot act as release hardware or raise a release gate.
 */
export function evaluateModuleRegressionFloor({
  environmentKind,
  observations,
  floors,
}) {
  if (!["floating-ci", "fixed-native", "experimental-local"].includes(environmentKind)) {
    throw new TypeError(
      "environmentKind must be floating-ci, fixed-native, or experimental-local",
    );
  }
  if (!Array.isArray(observations) || observations.length === 0) {
    throw new TypeError("observations must be a non-empty array");
  }
  if (!Array.isArray(floors) || floors.length === 0) {
    throw new TypeError("floors must be a non-empty array");
  }

  const breaches = [];
  for (const floor of floors) {
    if (!String(floor?.benchmark_id ?? "").trim()) {
      throw new TypeError("each floor requires benchmark_id");
    }
    if (!isFiniteNumber(floor?.p95_ceiling_nanoseconds) || floor.p95_ceiling_nanoseconds <= 0) {
      throw new TypeError("each floor requires a positive p95_ceiling_nanoseconds");
    }
    if (floor.release_gate === true) {
      if (environmentKind === "floating-ci") {
        throw new Error(
          "floating CI cannot evaluate release-gating regression floors; use a fixed native environment",
        );
      }
      if (floor.on_breach !== "block_release") {
        throw new Error("a release-gating floor must set on_breach to block_release");
      }
    }

    const observation = observations.find(
      (candidate) => candidate?.benchmark_id === floor.benchmark_id,
    );
    if (!observation) {
      breaches.push({
        benchmark_id: floor.benchmark_id,
        code: "PERFORMANCE_REGRESSION_OBSERVATION_MISSING",
      });
      continue;
    }
    if (!isFiniteNumber(observation.p95) || observation.p95 < 0) {
      throw new TypeError(`observation ${floor.benchmark_id} requires a finite non-negative p95`);
    }
    if (observation.p95 > floor.p95_ceiling_nanoseconds) {
      breaches.push({
        benchmark_id: floor.benchmark_id,
        code: "PERFORMANCE_REGRESSION_FLOOR_BREACHED",
        observed_p95: observation.p95,
        ceiling_p95: floor.p95_ceiling_nanoseconds,
        on_breach: floor.on_breach ?? "record_only",
      });
    }
  }

  return {
    claim_level: "hypothesis",
    environment_kind: environmentKind,
    release_hardware_evidence: environmentKind === "fixed-native",
    breaches,
    non_claims: [
      "floating CI is not release hardware evidence",
      "module regression floors are not Gate or Profile evidence",
      "does not claim Agent benefit",
    ],
  };
}

/**
 * Assemble a digest-bound governance A/B non-inferiority campaign report.
 * Requires owner-approved preregistration and rejects significant-benefit claims.
 */
export function buildGovernanceAbCampaignReport(campaign) {
  if (!campaign || typeof campaign !== "object" || Array.isArray(campaign)) {
    throw new TypeError("campaign must be an object");
  }
  if (campaign.claim_level !== "non_inferiority") {
    throw new Error("governance A/B campaign claim_level must be non_inferiority");
  }
  if (!String(campaign.preregistration_ref ?? "").trim()) {
    throw new Error("governance A/B campaign requires preregistration_ref");
  }
  if (!/^sha256:[a-f0-9]{64}$/.test(String(campaign.source_revision_digest ?? ""))) {
    // allow full git sha as source_revision separately
  }
  if (
    typeof campaign.source_revision !== "string" ||
    !/^[0-9a-f]{40}$/.test(campaign.source_revision)
  ) {
    throw new Error("governance A/B campaign requires a 40-character source_revision");
  }
  if (!/^sha256:[a-f0-9]{64}$/.test(String(campaign.environment_digest ?? ""))) {
    throw new Error("governance A/B campaign requires environment_digest");
  }
  if (campaign.environment_kind !== "fixed-native") {
    throw new Error("governance A/B campaign environment_kind must be fixed-native");
  }

  const denominator = campaign.denominator;
  if (
    !denominator ||
    !Number.isSafeInteger(denominator.started_attempts) ||
    denominator.started_attempts < 1 ||
    denominator.retained_attempts !== denominator.started_attempts
  ) {
    throw new Error(
      "governance A/B campaign requires complete started/retained attempt denominator",
    );
  }
  if (
    !campaign.safety ||
    campaign.safety.critical_safety_failures !== 0 ||
    campaign.safety.false_completions !== 0
  ) {
    throw new Error(
      "governance A/B campaign requires zero critical safety failures and zero false completions",
    );
  }

  const report = {
    schema_version: "cognitiveos.performance-report/0.1",
    benchmark_manifest: {
      workload: { name: "p7-t04-governance-ab" },
      samples: denominator.started_attempts,
      execution_state: "warm",
      environment_kind: "fixed-native",
      source_revision: campaign.source_revision,
      environment_digest: campaign.environment_digest,
      risk_class: "R1",
    },
    slo_profile: {
      id: "p7-t04-governance-ab",
      version: "1",
      window: "measured",
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
        p50: campaign.metrics.governed_latency_ms.p50,
        p95: campaign.metrics.governed_latency_ms.p95,
        p99: campaign.metrics.governed_latency_ms.p99,
        sample_count: denominator.started_attempts,
        confidence_interval: campaign.metrics.governed_latency_ms.confidence_interval,
      },
    ],
    comparison: {
      arms: [
        { arm_id: "A", arm_kind: "native_baseline" },
        { arm_id: "B", arm_kind: "governance_only" },
      ],
      claim_level: "non_inferiority",
      preregistration_ref: campaign.preregistration_ref,
      results: [
        {
          arm_a: "A",
          arm_b: "B",
          confidence_interval: campaign.comparison_confidence_interval,
        },
      ],
    },
    safety_failures: [],
    non_claims: [
      "not a significant-benefit claim",
      "not a Gate pass",
      "not a Profile claim",
      "does not block or pass GMVP-LINUX",
      "B06/B07 remain observations only",
    ],
  };

  const policyErrors = validatePerformanceReportPolicy(report);
  if (policyErrors.length > 0) {
    throw new Error(
      `governance A/B campaign failed policy validation: ${policyErrors
        .map((error) => error.code)
        .join(",")}`,
    );
  }

  const reportDigest = `sha256:${createHash("sha256")
    .update(JSON.stringify(report))
    .digest("hex")}`;
  return { report, report_digest: reportDigest };
}

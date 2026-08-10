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

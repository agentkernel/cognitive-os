import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";

const UCR_SCENARIO_ID = "UCR-01";
const REPORT_SCHEMA_VERSION = "cognitiveos.ucr-run-report/0.1";
const DIGEST_PREFIX = "CognitiveOS-Digest-V1\n";
const REPORT_DOMAIN = "cognitiveos.personal.ucr-run-report/0.1";
const REQUIRED_RESOURCE_FAMILIES = ["memory", "skill", "tool", "context", "task", "runtime"];
const PROHIBITED_CLAIM_KEYS = ["gate", "release", "profile", "completion", "passed"];
const B03_REQUIRED_OBSERVATIONS = [
  "authorized_context_only",
  "current_source_versions_only",
  "required_source_present",
  "no_false_completion",
];

function canonicalizeJson(value) {
  if (value === null || typeof value === "boolean" || typeof value === "string") {
    return JSON.stringify(value);
  }
  if (typeof value === "number") {
    if (!Number.isFinite(value)) throw new Error("measurements must be finite numbers");
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) return `[${value.map(canonicalizeJson).join(",")}]`;
  if (typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonicalizeJson(value[key])}`)
      .join(",")}}`;
  }
  throw new Error(`unsupported measurement value type: ${typeof value}`);
}

function digestJson(value) {
  return `sha256:${createHash("sha256")
    .update(DIGEST_PREFIX, "utf8")
    .update(REPORT_DOMAIN, "utf8")
    .update(Buffer.from([0]))
    .update(canonicalizeJson(value), "utf8")
    .digest("hex")}`;
}

function requireObject(value, fieldName) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${fieldName} must be an object`);
  }
  return value;
}

function requireDigest(value, fieldName) {
  if (typeof value !== "string" || !/^sha256:[0-9a-f]{64}$/.test(value)) {
    throw new Error(`${fieldName} must be a sha256 digest`);
  }
}

function rejectAuthorityClaims(value, fieldPath = "input") {
  if (Array.isArray(value)) {
    value.forEach((item, index) => rejectAuthorityClaims(item, `${fieldPath}[${index}]`));
    return;
  }
  if (value === null || typeof value !== "object") return;
  for (const [key, nestedValue] of Object.entries(value)) {
    if (PROHIBITED_CLAIM_KEYS.includes(key.toLowerCase())) {
      throw new Error(`${fieldPath}.${key} is forbidden: UCR raw runs are non-claim evidence`);
    }
    rejectAuthorityClaims(nestedValue, `${fieldPath}.${key}`);
  }
}

function validateMeasurements(measurements) {
  const measurementObject = requireObject(measurements, "measurements");
  for (const stratum of ["stable", "changed", "full_replay"]) {
    const tokenCount = measurementObject[stratum]?.repeated_input_tokens;
    if (!Number.isSafeInteger(tokenCount) || tokenCount < 0) {
      throw new Error(`measurements.${stratum}.repeated_input_tokens must be a non-negative integer`);
    }
  }
  if (!Number.isSafeInteger(measurementObject.tool_calls) || measurementObject.tool_calls < 0) {
    throw new Error("measurements.tool_calls must be a non-negative integer");
  }
  if (!Number.isSafeInteger(measurementObject.tool_failures) || measurementObject.tool_failures < 0) {
    throw new Error("measurements.tool_failures must be a non-negative integer");
  }
}

export function buildUcrRunReport(rawRun, stableBaseline) {
  const run = requireObject(rawRun, "raw run");
  const baseline = requireObject(stableBaseline, "stable baseline");
  rejectAuthorityClaims(run);
  if (run.scenario_id !== UCR_SCENARIO_ID) throw new Error("raw run must identify UCR-01");
  if (run.claim_scope !== "non-claim") throw new Error("raw run claim_scope must be non-claim");
  if (!Array.isArray(run.resource_families)) throw new Error("raw run resource_families must be an array");
  if (
    [...run.resource_families].sort().join(",") !==
    [...REQUIRED_RESOURCE_FAMILIES].sort().join(",")
  ) {
    throw new Error("raw run must cover each UCR-01 resource family exactly once");
  }
  for (const digestField of ["fixture_digest", "trace_digest", "baseline_digest"]) {
    requireDigest(run[digestField], `raw run ${digestField}`);
  }
  validateMeasurements(run.measurements);
  requireDigest(baseline.baseline_digest, "stable baseline baseline_digest");
  if (baseline.baseline_digest !== run.baseline_digest) {
    throw new Error("raw run baseline_digest does not match the pinned stable baseline");
  }
  validateMeasurements(baseline.measurements);

  const report = {
    schema_version: REPORT_SCHEMA_VERSION,
    scenario_id: UCR_SCENARIO_ID,
    claim_scope: "non-claim",
    fixture_digest: run.fixture_digest,
    trace_digest: run.trace_digest,
    baseline_digest: run.baseline_digest,
    resource_families: REQUIRED_RESOURCE_FAMILIES,
    measurements: run.measurements,
    baseline_measurements: baseline.measurements,
    observations: {
      stable_repeated_input_delta: run.measurements.stable.repeated_input_tokens - baseline.measurements.stable.repeated_input_tokens,
      changed_repeated_input_delta: run.measurements.changed.repeated_input_tokens - baseline.measurements.changed.repeated_input_tokens,
    },
  };
  return { report, report_digest: digestJson(report) };
}

export function buildB06B07ObservationReport(campaign) {
  const campaignDocument = requireObject(campaign, "B06/B07 campaign");
  rejectAuthorityClaims(campaignDocument);
  if (campaignDocument.campaign_id !== "B06-B07-context-benefit-observation/1") {
    throw new Error("B06/B07 campaign_id must be B06-B07-context-benefit-observation/1");
  }
  if (campaignDocument.claim_scope !== "non-claim") {
    throw new Error("B06/B07 campaign claim_scope must be non-claim");
  }
  if (campaignDocument.scenario_id !== UCR_SCENARIO_ID) {
    throw new Error("B06/B07 campaign must bind UCR-01");
  }
  for (const digestField of ["fixture_digest", "trace_digest", "governed_stage_digest"]) {
    requireDigest(campaignDocument[digestField], `B06/B07 ${digestField}`);
  }

  const denominator = requireObject(campaignDocument.denominator, "B06/B07 denominator");
  const startedAttempts = denominator.started_attempts;
  const retainedAttempts = denominator.retained_attempts;
  if (
    !Number.isSafeInteger(startedAttempts) ||
    !Number.isSafeInteger(retainedAttempts) ||
    startedAttempts < 1 ||
    retainedAttempts !== startedAttempts
  ) {
    throw new Error(
      "B06/B07 denominator requires started_attempts >= 1 and retained_attempts equal to started_attempts",
    );
  }

  const safety = requireObject(campaignDocument.safety, "B06/B07 safety");
  for (const safetyField of ["critical_safety_failures", "false_completions"]) {
    const value = safety[safetyField];
    if (!Number.isSafeInteger(value) || value < 0) {
      throw new Error(`B06/B07 safety.${safetyField} must be a non-negative integer`);
    }
  }
  if (safety.critical_safety_failures !== 0 || safety.false_completions !== 0) {
    throw new Error(
      "B06/B07 observations require zero critical safety failures and zero false completions",
    );
  }

  const strata = requireObject(campaignDocument.strata, "B06/B07 strata");
  const fullReplay = requireObject(strata.full_replay, "B06/B07 strata.full_replay");
  const stable = requireObject(strata.stable, "B06/B07 strata.stable");
  const changed = requireObject(strata.changed, "B06/B07 strata.changed");
  for (const [name, stratum] of [
    ["full_replay", fullReplay],
    ["stable", stable],
    ["changed", changed],
  ]) {
    if (
      !Number.isSafeInteger(stratum.repeated_input_tokens) ||
      stratum.repeated_input_tokens < 1
    ) {
      throw new Error(`B06/B07 strata.${name}.repeated_input_tokens must be a positive integer`);
    }
    if (
      typeof stratum.verified_completion_rate !== "number" ||
      !Number.isFinite(stratum.verified_completion_rate) ||
      stratum.verified_completion_rate < 0 ||
      stratum.verified_completion_rate > 1
    ) {
      throw new Error(
        `B06/B07 strata.${name}.verified_completion_rate must be a finite number in [0, 1]`,
      );
    }
  }

  function reduction(optimizedTokens) {
    return Number(
      (
        (fullReplay.repeated_input_tokens - optimizedTokens) /
        fullReplay.repeated_input_tokens
      ).toFixed(6),
    );
  }

  const stableReduction = reduction(stable.repeated_input_tokens);
  const changedReduction = reduction(changed.repeated_input_tokens);
  if (
    stable.verified_completion_rate < fullReplay.verified_completion_rate ||
    changed.verified_completion_rate < fullReplay.verified_completion_rate
  ) {
    throw new Error(
      "B06/B07 observations reject completion-rate regressions versus full replay",
    );
  }

  const report = {
    schema_version: "cognitiveos.b06-b07-observation-report/0.1",
    campaign_id: campaignDocument.campaign_id,
    scenario_id: UCR_SCENARIO_ID,
    claim_scope: "non-claim",
    fixture_digest: campaignDocument.fixture_digest,
    trace_digest: campaignDocument.trace_digest,
    governed_stage_digest: campaignDocument.governed_stage_digest,
    denominator: {
      started_attempts: startedAttempts,
      retained_attempts: retainedAttempts,
    },
    safety: {
      critical_safety_failures: safety.critical_safety_failures,
      false_completions: safety.false_completions,
    },
    observations: {
      B06_stable_context: {
        repeated_input_tokens: stable.repeated_input_tokens,
        full_replay_repeated_input_tokens: fullReplay.repeated_input_tokens,
        repeated_input_reduction: stableReduction,
        meets_twenty_percent_reduction_observation: stableReduction >= 0.2,
        verified_completion_rate: stable.verified_completion_rate,
      },
      B07_changed_context: {
        repeated_input_tokens: changed.repeated_input_tokens,
        full_replay_repeated_input_tokens: fullReplay.repeated_input_tokens,
        repeated_input_reduction: changedReduction,
        meets_twenty_percent_reduction_observation: changedReduction >= 0.2,
        verified_completion_rate: changed.verified_completion_rate,
      },
    },
    non_claims: [
      "not a Gate pass",
      "not a release claim",
      "not a Profile claim",
      "not a generalized Agent-benefit claim",
      "does not block GMVP-LINUX",
    ],
  };
  return { report, report_digest: digestJson(report) };
}

export function buildB03ObservationReport(campaign) {
  const campaignDocument = requireObject(campaign, "B03 campaign");
  rejectAuthorityClaims(campaignDocument);
  if (campaignDocument.campaign_id !== "B03-context-correctness/1") {
    throw new Error("B03 campaign_id must be B03-context-correctness/1");
  }
  if (campaignDocument.claim_scope !== "non-claim") {
    throw new Error("B03 campaign claim_scope must be non-claim");
  }
  requireDigest(campaignDocument.context_view_digest, "B03 context_view_digest");
  const observations = requireObject(campaignDocument.observations, "B03 observations");
  for (const observationName of B03_REQUIRED_OBSERVATIONS) {
    if (observations[observationName] !== true) {
      throw new Error(`B03 observation ${observationName} must be explicitly true`);
    }
  }
  const report = {
    schema_version: "cognitiveos.b03-observation-report/0.1",
    campaign_id: campaignDocument.campaign_id,
    claim_scope: "non-claim",
    context_view_digest: campaignDocument.context_view_digest,
    observations: B03_REQUIRED_OBSERVATIONS,
  };
  return { report, report_digest: digestJson(report) };
}

export function runUcrCli(argumentsList) {
  const [rawRunPath, stableBaselinePath, outputPath] = argumentsList;
  if (!rawRunPath || !stableBaselinePath || !outputPath) {
    throw new Error("usage: node src/ucr-runner.mjs <raw-run.json> <stable-baseline.json> <output.json>");
  }
  const result = buildUcrRunReport(
    JSON.parse(readFileSync(rawRunPath, "utf8")),
    JSON.parse(readFileSync(stableBaselinePath, "utf8")),
  );
  writeFileSync(outputPath, `${JSON.stringify(result, null, 2)}\n`);
}

if (process.argv[1]?.endsWith("ucr-runner.mjs")) {
  runUcrCli(process.argv.slice(2));
}

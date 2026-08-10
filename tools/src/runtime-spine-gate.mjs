import { createHash } from "node:crypto";

const DIGEST_PREFIX = "CognitiveOS-Digest-V1\n";
const REPORT_DOMAIN = "cognitiveos.personal.runtime-spine-gate-report/0.1";
const REPORT_SCHEMA_VERSION = "cognitiveos.runtime-spine-gate-report/0.1";
const CAMPAIGN_ID = "runtime-spine-gates/1";
const PROHIBITED_CLAIM_KEYS = ["gate", "release", "profile", "completion", "passed"];

export const RUNTIME_SPINE_REQUIRED_OBSERVATIONS = Object.freeze([
  "six_family_projection_isolated",
  "task_management_channel_isolated",
  "default_path_confirmation_recorded",
  "tier2_purge_requires_explicit_confirmation",
  "shell_close_preserved_authority",
  "daemon_close_recoverable",
  "outcome_unknown_reconciled_by_original_key",
  "no_blind_retry_without_key_change",
  "no_false_completion",
  "adr0018_local_native_exception_absent_or_replaced",
]);

function canonicalizeJson(value) {
  if (value === null || typeof value === "boolean" || typeof value === "string") {
    return JSON.stringify(value);
  }
  if (typeof value === "number") {
    if (!Number.isFinite(value)) throw new Error("values must be finite numbers");
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) return `[${value.map(canonicalizeJson).join(",")}]`;
  if (typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonicalizeJson(value[key])}`)
      .join(",")}}`;
  }
  throw new Error(`unsupported value type: ${typeof value}`);
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
      throw new Error(
        `${fieldPath}.${key} is forbidden: Runtime Spine suite reports are non-claim evidence`,
      );
    }
    rejectAuthorityClaims(nestedValue, `${fieldPath}.${key}`);
  }
}

/**
 * Build a digest-bound non-claim Runtime Spine Gate suite report.
 *
 * The evaluator never sets B02/B04/B05/B12 Gate state. It only accepts a
 * complete observation set for later campaign execution review.
 */
export function buildRuntimeSpineGateSuiteReport(campaign) {
  const campaignDocument = requireObject(campaign, "Runtime Spine campaign");
  rejectAuthorityClaims(campaignDocument);
  if (campaignDocument.campaign_id !== CAMPAIGN_ID) {
    throw new Error(`Runtime Spine campaign_id must be ${CAMPAIGN_ID}`);
  }
  if (campaignDocument.claim_scope !== "non-claim") {
    throw new Error("Runtime Spine campaign claim_scope must be non-claim");
  }
  requireDigest(campaignDocument.suite_digest, "Runtime Spine suite_digest");
  requireDigest(campaignDocument.trace_digest, "Runtime Spine trace_digest");
  const targetGates = campaignDocument.target_gates;
  if (
    !Array.isArray(targetGates) ||
    [...targetGates].sort().join(",") !== ["B02", "B04", "B05", "B12"].join(",")
  ) {
    throw new Error("Runtime Spine target_gates must be exactly [B02, B04, B05, B12]");
  }
  const observations = requireObject(campaignDocument.observations, "Runtime Spine observations");
  for (const observationName of RUNTIME_SPINE_REQUIRED_OBSERVATIONS) {
    if (observations[observationName] !== true) {
      throw new Error(`Runtime Spine observation ${observationName} must be explicitly true`);
    }
  }
  const confirmationCount = campaignDocument.default_path_confirmation_count;
  if (!Number.isSafeInteger(confirmationCount) || confirmationCount < 0) {
    throw new Error("default_path_confirmation_count must be a non-negative integer");
  }
  if (confirmationCount > 1) {
    throw new Error(
      "default_path_confirmation_count must be <= 1 for the default B04 path (Tier-2 excluded)",
    );
  }

  const report = {
    schema_version: REPORT_SCHEMA_VERSION,
    campaign_id: CAMPAIGN_ID,
    claim_scope: "non-claim",
    target_gates: ["B02", "B04", "B05", "B12"],
    suite_digest: campaignDocument.suite_digest,
    trace_digest: campaignDocument.trace_digest,
    default_path_confirmation_count: confirmationCount,
    observations: [...RUNTIME_SPINE_REQUIRED_OBSERVATIONS],
    non_claims: [
      "not a B02/B04/B05/B12 Gate pass",
      "not a release claim",
      "not a Profile claim",
      "does not set Gate state",
    ],
  };
  return { report, report_digest: digestJson(report) };
}

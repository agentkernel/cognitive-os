import { createHash } from "node:crypto";

const DIGEST_PREFIX = "CognitiveOS-Digest-V1\n";
const REPORT_DOMAIN = "cognitiveos.personal.b10-dynamic-tool-gate-report/0.1";
const REPORT_SCHEMA_VERSION = "cognitiveos.b10-dynamic-tool-gate-report/0.1";
const CAMPAIGN_ID = "B10-dynamic-tool-ecosystem/1";
const PROHIBITED_CLAIM_KEYS = ["gate", "release", "profile", "completion", "passed"];

export const B10_REQUIRED_OBSERVATIONS = Object.freeze([
  "dynamic_package_identity_bound",
  "discovery_disabled_no_auto_enable",
  "task_contract_scoped_exposure",
  "enable_requires_requalification",
  "disable_removes_exposure",
  "quarantine_blocks_enable",
  "package_manifest_drift_fail_closed",
  "reconcile_unknown_outcome_original_key",
  "composite_retains_child_intent_effect",
  "pure_read_cache_only",
  "sandbox_bypass_rejected",
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
        `${fieldPath}.${key} is forbidden: B10 suite reports are non-claim evidence`,
      );
    }
    rejectAuthorityClaims(nestedValue, `${fieldPath}.${key}`);
  }
}

/**
 * Build a digest-bound non-claim B10 dynamic Tool Gate suite report.
 *
 * The evaluator never sets B10 Gate state. It only accepts a complete
 * observation set for later campaign execution review.
 */
export function buildB10DynamicToolGateSuiteReport(campaign) {
  const campaignDocument = requireObject(campaign, "B10 campaign");
  rejectAuthorityClaims(campaignDocument);
  if (campaignDocument.campaign_id !== CAMPAIGN_ID) {
    throw new Error(`B10 campaign_id must be ${CAMPAIGN_ID}`);
  }
  if (campaignDocument.claim_scope !== "non-claim") {
    throw new Error("B10 campaign claim_scope must be non-claim");
  }
  requireDigest(campaignDocument.suite_digest, "B10 suite_digest");
  requireDigest(campaignDocument.trace_digest, "B10 trace_digest");
  const targetGates = campaignDocument.target_gates;
  if (
    !Array.isArray(targetGates) ||
    targetGates.length !== 1 ||
    targetGates[0] !== "B10"
  ) {
    throw new Error("B10 target_gates must be exactly [B10]");
  }
  const observations = requireObject(campaignDocument.observations, "B10 observations");
  for (const observationName of B10_REQUIRED_OBSERVATIONS) {
    if (observations[observationName] !== true) {
      throw new Error(`B10 observation ${observationName} must be explicitly true`);
    }
  }
  const report = {
    schema_version: REPORT_SCHEMA_VERSION,
    campaign_id: CAMPAIGN_ID,
    claim_scope: "non-claim",
    target_gates: ["B10"],
    suite_digest: campaignDocument.suite_digest,
    trace_digest: campaignDocument.trace_digest,
    observations: [...B10_REQUIRED_OBSERVATIONS],
    non_claims: [
      "does not set Gate state",
      "does not claim automatic marketplace discovery enablement",
      "does not claim GMVP-LINUX, release, or Profile",
      "does not create public Tool schema authority",
    ],
  };
  return { report, report_digest: digestJson(report) };
}

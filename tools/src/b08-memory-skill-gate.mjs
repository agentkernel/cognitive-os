import { createHash } from "node:crypto";

const DIGEST_PREFIX = "CognitiveOS-Digest-V1\n";
const REPORT_DOMAIN = "cognitiveos.personal.b08-memory-skill-gate-report/0.1";
const REPORT_SCHEMA_VERSION = "cognitiveos.b08-memory-skill-gate-report/0.1";
const CAMPAIGN_ID = "B08-memory-skill-consumption/1";
const PROHIBITED_CLAIM_KEYS = ["gate", "release", "profile", "completion", "passed"];

export const B08_REQUIRED_OBSERVATIONS = Object.freeze([
  "memory_admission_current_source",
  "memory_stale_source_rejects",
  "memory_reject_decision_no_object",
  "memory_search_authority_filter",
  "memory_forget_no_resurrection",
  "memory_expiry_boundary",
  "memory_version_cas_supersede",
  "skill_workspace_binding",
  "skill_unsafe_revoke_fail_closed",
  "skill_supersede_exact_pins",
  "task_consumption_channel_isolation",
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
        `${fieldPath}.${key} is forbidden: B08 suite reports are non-claim evidence`,
      );
    }
    rejectAuthorityClaims(nestedValue, `${fieldPath}.${key}`);
  }
}

/**
 * Build a digest-bound non-claim B08 Memory/Skill Gate suite report.
 *
 * The evaluator never sets B08 Gate state. It only accepts a complete
 * observation set for later campaign execution review.
 */
export function buildB08MemorySkillGateSuiteReport(campaign) {
  const campaignDocument = requireObject(campaign, "B08 campaign");
  rejectAuthorityClaims(campaignDocument);
  if (campaignDocument.campaign_id !== CAMPAIGN_ID) {
    throw new Error(`B08 campaign_id must be ${CAMPAIGN_ID}`);
  }
  if (campaignDocument.claim_scope !== "non-claim") {
    throw new Error("B08 campaign claim_scope must be non-claim");
  }
  requireDigest(campaignDocument.suite_digest, "B08 suite_digest");
  requireDigest(campaignDocument.trace_digest, "B08 trace_digest");
  const targetGates = campaignDocument.target_gates;
  if (
    !Array.isArray(targetGates) ||
    targetGates.length !== 1 ||
    targetGates[0] !== "B08"
  ) {
    throw new Error("B08 target_gates must be exactly [B08]");
  }
  const observations = requireObject(campaignDocument.observations, "B08 observations");
  for (const observationName of B08_REQUIRED_OBSERVATIONS) {
    if (observations[observationName] !== true) {
      throw new Error(`B08 observation ${observationName} must be explicitly true`);
    }
  }
  const report = {
    schema_version: REPORT_SCHEMA_VERSION,
    campaign_id: CAMPAIGN_ID,
    claim_scope: "non-claim",
    target_gates: ["B08"],
    suite_digest: campaignDocument.suite_digest,
    trace_digest: campaignDocument.trace_digest,
    observations: [...B08_REQUIRED_OBSERVATIONS],
    non_claims: [
      "does not set Gate state",
      "does not claim embedding/vector/graph retrieval",
      "does not claim GMVP-LINUX, release, or Profile",
      "does not create public Memory/Skill schema authority",
    ],
  };
  return { report, report_digest: digestJson(report) };
}

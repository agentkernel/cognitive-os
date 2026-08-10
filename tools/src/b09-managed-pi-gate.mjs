import { createHash } from "node:crypto";

const DIGEST_PREFIX = "CognitiveOS-Digest-V1\n";
const REPORT_DOMAIN = "cognitiveos.personal.b09-managed-pi-gate-report/0.1";
const REPORT_SCHEMA_VERSION = "cognitiveos.b09-managed-pi-gate-report/0.1";
const CAMPAIGN_ID = "B09-managed-pi-sidecar/1";
const PROHIBITED_CLAIM_KEYS = ["gate", "release", "profile", "completion", "passed"];

export const B09_REQUIRED_OBSERVATIONS = Object.freeze([
  "process_bound_on_activate",
  "unbound_registered_health",
  "pause_stop_clear_binding",
  "stale_epoch_preserves_binding",
  "process_bound_blocks_upgrade",
  "process_bound_blocks_uninstall",
  "pin_drift_rejects_activation",
  "stop_then_uninstall",
  "install_neq_permission",
  "identity_separation",
  "orphan_no_reattach",
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
        `${fieldPath}.${key} is forbidden: B09 suite reports are non-claim evidence`,
      );
    }
    rejectAuthorityClaims(nestedValue, `${fieldPath}.${key}`);
  }
}

/**
 * Build a digest-bound non-claim B09 managed-Pi Gate suite report.
 *
 * The evaluator never sets B09 Gate state. It only accepts a complete
 * observation set for later campaign execution review.
 */
export function buildB09ManagedPiGateSuiteReport(campaign) {
  const campaignDocument = requireObject(campaign, "B09 campaign");
  rejectAuthorityClaims(campaignDocument);
  if (campaignDocument.campaign_id !== CAMPAIGN_ID) {
    throw new Error(`B09 campaign_id must be ${CAMPAIGN_ID}`);
  }
  if (campaignDocument.claim_scope !== "non-claim") {
    throw new Error("B09 campaign claim_scope must be non-claim");
  }
  requireDigest(campaignDocument.suite_digest, "B09 suite_digest");
  requireDigest(campaignDocument.trace_digest, "B09 trace_digest");
  const targetGates = campaignDocument.target_gates;
  if (
    !Array.isArray(targetGates) ||
    targetGates.length !== 1 ||
    targetGates[0] !== "B09"
  ) {
    throw new Error("B09 target_gates must be exactly [B09]");
  }
  const observations = requireObject(campaignDocument.observations, "B09 observations");
  for (const observationName of B09_REQUIRED_OBSERVATIONS) {
    if (observations[observationName] !== true) {
      throw new Error(`B09 observation ${observationName} must be explicitly true`);
    }
  }
  const report = {
    schema_version: REPORT_SCHEMA_VERSION,
    campaign_id: CAMPAIGN_ID,
    claim_scope: "non-claim",
    target_gates: ["B09"],
    suite_digest: campaignDocument.suite_digest,
    trace_digest: campaignDocument.trace_digest,
    observations: [...B09_REQUIRED_OBSERVATIONS],
    non_claims: [
      "does not set Gate state",
      "does not qualify non-Pi adapters",
      "does not claim GMVP-LINUX, release, or Profile",
      "does not claim live production process supervision",
    ],
  };
  return { report, report_digest: digestJson(report) };
}

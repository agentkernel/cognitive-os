import { createHash } from "node:crypto";

const DIGEST_PREFIX = "CognitiveOS-Digest-V1\n";
const REPORT_DOMAIN = "cognitiveos.personal.gmvp-linux-gate-report/0.1";
const REPORT_SCHEMA_VERSION = "cognitiveos.gmvp-linux-gate-report/0.1";
const CAMPAIGN_ID = "GMVP-LINUX-composition/1";
const PROHIBITED_CLAIM_KEYS = ["gate", "release", "profile", "completion", "passed"];

/** Promotion Gate composition exact set for GMVP-LINUX (B06/B07/B10/B11 excluded). */
export const GMVP_REQUIRED_GATE_OBSERVATIONS = Object.freeze([
  "b01_mvp_pass",
  "b02_mvp_pass",
  "b03_mvp_pass",
  "b04_mvp_pass",
  "b05_mvp_pass",
  "b08_mvp_pass",
  "b09_mvp_pass",
  "b12_mvp_pass",
]);

/** UCR-01 fixed-scenario acceptance assertions bound into P7-T08 composition. */
export const GMVP_REQUIRED_UCR_ASSERTIONS = Object.freeze([
  "required_recall",
  "no_unauthorized_stale_exposure",
  "skill_reuse",
  "no_duplicate_effect",
  "no_false_completion",
  "stale_epoch_rejected",
  "stable_changed_context_token_reduction",
]);

/** Product operability evidence rollup (already delivered by P7-T01..T03 / B09). */
export const GMVP_REQUIRED_OPERABILITY = Object.freeze([
  "six_resource_release_manifest",
  "sbom_attestation_digest_bound",
  "lifecycle_backup_restore",
  "six_resource_doctor",
  "headless_vault_doctor",
  "desktop_or_headless_secretstore_path",
  "pi_sidecar_b09_pins",
]);

export const GMVP_REQUIRED_OBSERVATIONS = Object.freeze([
  ...GMVP_REQUIRED_GATE_OBSERVATIONS,
  ...GMVP_REQUIRED_UCR_ASSERTIONS,
  ...GMVP_REQUIRED_OPERABILITY,
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
        `${fieldPath}.${key} is forbidden: GMVP-LINUX suite reports are non-claim evidence`,
      );
    }
    rejectAuthorityClaims(nestedValue, `${fieldPath}.${key}`);
  }
}

/**
 * Build a digest-bound non-claim GMVP-LINUX composition report.
 *
 * The evaluator never sets GMVP-LINUX Gate state. It only accepts a complete
 * composition observation set for later disposition review.
 */
export function buildGmvpLinuxGateSuiteReport(campaign) {
  const campaignDocument = requireObject(campaign, "GMVP campaign");
  rejectAuthorityClaims(campaignDocument);
  if (campaignDocument.campaign_id !== CAMPAIGN_ID) {
    throw new Error(`GMVP campaign_id must be ${CAMPAIGN_ID}`);
  }
  if (campaignDocument.claim_scope !== "non-claim") {
    throw new Error("GMVP campaign claim_scope must be non-claim");
  }
  requireDigest(campaignDocument.suite_digest, "GMVP suite_digest");
  requireDigest(campaignDocument.trace_digest, "GMVP trace_digest");
  const targetGates = campaignDocument.target_gates;
  if (
    !Array.isArray(targetGates) ||
    targetGates.length !== 1 ||
    targetGates[0] !== "GMVP-LINUX"
  ) {
    throw new Error("GMVP target_gates must be exactly [GMVP-LINUX]");
  }
  const observations = requireObject(campaignDocument.observations, "GMVP observations");
  for (const observationName of GMVP_REQUIRED_OBSERVATIONS) {
    if (observations[observationName] !== true) {
      throw new Error(`GMVP observation ${observationName} must be explicitly true`);
    }
  }
  const report = {
    schema_version: REPORT_SCHEMA_VERSION,
    campaign_id: CAMPAIGN_ID,
    claim_scope: "non-claim",
    target_gates: ["GMVP-LINUX"],
    suite_digest: campaignDocument.suite_digest,
    trace_digest: campaignDocument.trace_digest,
    observations: [...GMVP_REQUIRED_OBSERVATIONS],
    non_claims: [
      "does not set Gate state",
      "does not claim Profile conformance",
      "does not claim B06/B07/B10/B11 benefit",
      "does not claim Windows install parity (B01-W)",
    ],
  };
  return { report, report_digest: digestJson(report) };
}

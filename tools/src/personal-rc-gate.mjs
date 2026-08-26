import { createHash } from "node:crypto";

const DIGEST_PREFIX = "CognitiveOS-Digest-V1\n";
const REPORT_DOMAIN = "cognitiveos.personal.rc-declaration-report/0.1";
const REPORT_SCHEMA_VERSION = "cognitiveos.personal-rc-declaration-report/0.1";
const CAMPAIGN_ID = "PERSONAL-LINUX-RC-declaration/1";
const CLAIM_SCOPE = "personal-linux-rc-declaration";
const P6_DISPOSITION = "disabled-nogo";
const PROHIBITED_CLAIM_KEYS = [
  "profile",
  "passed",
  "github_release_published",
  "production_signing",
  "implemented",
];

/** Declared-scope B01–B12 plus GMVP-LINUX composition, bound to existing MVP evidence. */
export const RC_REQUIRED_GATE_OBSERVATIONS = Object.freeze([
  "b01_mvp_pass",
  "b02_mvp_pass",
  "b03_mvp_pass",
  "b04_mvp_pass",
  "b05_mvp_pass",
  "b08_mvp_pass",
  "b09_mvp_pass",
  "b12_mvp_pass",
  "gmvp_linux_mvp_pass",
]);

/** RC operability: CI, SBOM/attestation, lifecycle, support matrix, runbooks, clean-VM suite. */
export const RC_REQUIRED_OPERABILITY = Object.freeze([
  "required_ci_both_platforms",
  "six_resource_release_manifest",
  "sbom_attestation_digest_bound",
  "lifecycle_update_rollback_uninstall",
  "support_matrix_matches_claim_set",
  "runbooks_published",
  "clean_vm_suite_bound",
]);

export const RC_REQUIRED_EVIDENCE_OBSERVATIONS = Object.freeze([
  ...RC_REQUIRED_GATE_OBSERVATIONS,
  ...RC_REQUIRED_OPERABILITY,
]);

/** Explicit exclusions that must be recorded true (meaning: stated, not enabled). */
export const RC_REQUIRED_DISPOSITIONS = Object.freeze([
  "p6_disabled_nogo",
  "b06_b07_non_claim",
  "b10_not_in_rc_claim",
  "web_ui_non_blocking_not_in_rc",
  "windows_no_install_parity",
]);

export const RC_REQUIRED_EXPLICIT_NON_CLAIMS = Object.freeze([
  "does not set Gate state",
  "does not claim Profile conformance",
  "does not claim a production GitHub Release or production signing ceremony",
  "does not claim Windows install parity (B01-W)",
  "does not enable Multi-Agent / B11",
  "does not include B10/MCP/dynamic Tool in the Linux RC claim",
  "does not include Web UI in the Linux RC claim",
  "does not promote B06/B07 observations to a benefit or Gate pass",
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
        `${fieldPath}.${key} is forbidden: Personal Linux RC reports must not impersonate Profile, invent a pass, or claim production publication`,
      );
    }
    rejectAuthorityClaims(nestedValue, `${fieldPath}.${key}`);
  }
}

/**
 * Build a digest-bound Personal Linux RC declaration report.
 *
 * The evaluator never sets RC/Gate/Profile state. Every evidence observation
 * must carry an explicit sha256 binding. P6 must be recorded disabled-nogo.
 */
export function buildPersonalRcDeclarationReport(campaign) {
  const campaignDocument = requireObject(campaign, "RC campaign");
  rejectAuthorityClaims(campaignDocument);
  if (campaignDocument.campaign_id !== CAMPAIGN_ID) {
    throw new Error(`RC campaign_id must be ${CAMPAIGN_ID}`);
  }
  if (campaignDocument.claim_scope !== CLAIM_SCOPE) {
    throw new Error(`RC campaign claim_scope must be ${CLAIM_SCOPE}`);
  }
  if (campaignDocument.p6_disposition !== P6_DISPOSITION) {
    throw new Error(`RC p6_disposition must be ${P6_DISPOSITION}`);
  }
  if (campaignDocument.open_critical_risks_for_this_rc !== 0) {
    throw new Error("RC open_critical_risks_for_this_rc must be 0");
  }
  requireDigest(campaignDocument.suite_digest, "RC suite_digest");
  requireDigest(campaignDocument.trace_digest, "RC trace_digest");
  const targetGates = campaignDocument.target_gates;
  if (!Array.isArray(targetGates) || targetGates.length !== 1 || targetGates[0] !== "RC") {
    throw new Error("RC target_gates must be exactly [RC]");
  }
  const observations = requireObject(campaignDocument.observations, "RC observations");
  for (const observationName of RC_REQUIRED_EVIDENCE_OBSERVATIONS) {
    if (observations[observationName] !== true) {
      throw new Error(`RC observation ${observationName} must be explicitly true`);
    }
  }
  for (const dispositionName of RC_REQUIRED_DISPOSITIONS) {
    if (observations[dispositionName] !== true) {
      throw new Error(`RC disposition ${dispositionName} must be explicitly true`);
    }
  }
  const evidenceBindings = requireObject(
    campaignDocument.evidence_bindings,
    "RC evidence_bindings",
  );
  for (const observationName of RC_REQUIRED_EVIDENCE_OBSERVATIONS) {
    requireDigest(
      evidenceBindings[observationName],
      `RC evidence_bindings.${observationName}`,
    );
  }
  const explicitNonClaims = campaignDocument.explicit_non_claims;
  if (!Array.isArray(explicitNonClaims)) {
    throw new Error("RC explicit_non_claims must be an array");
  }
  for (const required of RC_REQUIRED_EXPLICIT_NON_CLAIMS) {
    if (!explicitNonClaims.includes(required)) {
      throw new Error(`RC explicit_non_claims must include ${required}`);
    }
  }
  const report = {
    schema_version: REPORT_SCHEMA_VERSION,
    campaign_id: CAMPAIGN_ID,
    claim_scope: CLAIM_SCOPE,
    p6_disposition: P6_DISPOSITION,
    open_critical_risks_for_this_rc: 0,
    target_gates: ["RC"],
    suite_digest: campaignDocument.suite_digest,
    trace_digest: campaignDocument.trace_digest,
    observations: [...RC_REQUIRED_EVIDENCE_OBSERVATIONS, ...RC_REQUIRED_DISPOSITIONS],
    evidence_bindings: Object.fromEntries(
      RC_REQUIRED_EVIDENCE_OBSERVATIONS.map((name) => [name, evidenceBindings[name]]),
    ),
    non_claims: [...RC_REQUIRED_EXPLICIT_NON_CLAIMS],
  };
  return { report, report_digest: digestJson(report) };
}

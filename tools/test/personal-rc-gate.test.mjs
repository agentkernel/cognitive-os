import assert from "node:assert/strict";
import test from "node:test";

import {
  RC_REQUIRED_DISPOSITIONS,
  RC_REQUIRED_EVIDENCE_OBSERVATIONS,
  RC_REQUIRED_EXPLICIT_NON_CLAIMS,
  buildPersonalRcDeclarationReport,
} from "../src/personal-rc-gate.mjs";

const digest = (character) => `sha256:${character.repeat(64)}`;

function campaign(overrides = {}) {
  const observations = Object.fromEntries([
    ...RC_REQUIRED_EVIDENCE_OBSERVATIONS.map((name) => [name, true]),
    ...RC_REQUIRED_DISPOSITIONS.map((name) => [name, true]),
  ]);
  const evidence_bindings = Object.fromEntries(
    RC_REQUIRED_EVIDENCE_OBSERVATIONS.map((name) => [name, digest("a")]),
  );
  return {
    campaign_id: "PERSONAL-LINUX-RC-declaration/1",
    claim_scope: "personal-linux-rc-declaration",
    p6_disposition: "disabled-nogo",
    open_critical_risks_for_this_rc: 0,
    target_gates: ["RC"],
    suite_digest: digest("c"),
    trace_digest: digest("d"),
    observations,
    evidence_bindings,
    explicit_non_claims: [...RC_REQUIRED_EXPLICIT_NON_CLAIMS],
    ...overrides,
  };
}

test("buildPersonalRcDeclarationReport records a complete digest-bound declaration", () => {
  const first = buildPersonalRcDeclarationReport(campaign());
  const second = buildPersonalRcDeclarationReport(campaign());

  assert.equal(first.report.claim_scope, "personal-linux-rc-declaration");
  assert.equal(first.report.p6_disposition, "disabled-nogo");
  assert.equal(first.report.open_critical_risks_for_this_rc, 0);
  assert.deepEqual(first.report.target_gates, ["RC"]);
  assert.equal(
    first.report.observations.length,
    RC_REQUIRED_EVIDENCE_OBSERVATIONS.length + RC_REQUIRED_DISPOSITIONS.length,
  );
  assert.equal(
    Object.keys(first.report.evidence_bindings).length,
    RC_REQUIRED_EVIDENCE_OBSERVATIONS.length,
  );
  assert.equal(first.report_digest, second.report_digest);
  assert.ok(first.report.non_claims.includes("does not claim Profile conformance"));
  assert.ok(first.report.non_claims.includes("does not set Gate state"));
});

test("buildPersonalRcDeclarationReport rejects incomplete, Profile, and P6-enabled campaigns", () => {
  const incomplete = campaign();
  incomplete.observations.clean_vm_suite_bound = false;
  assert.throws(() => buildPersonalRcDeclarationReport(incomplete), /clean_vm_suite_bound/);

  const missingDigest = campaign();
  delete missingDigest.evidence_bindings.sbom_attestation_digest_bound;
  assert.throws(
    () => buildPersonalRcDeclarationReport(missingDigest),
    /evidence_bindings.sbom_attestation_digest_bound/,
  );

  const claimed = campaign({ profile: "implemented" });
  assert.throws(() => buildPersonalRcDeclarationReport(claimed), /forbidden/);

  const github = campaign({ github_release_published: true });
  assert.throws(() => buildPersonalRcDeclarationReport(github), /forbidden/);

  const p6Enabled = campaign({ p6_disposition: "enabled" });
  assert.throws(() => buildPersonalRcDeclarationReport(p6Enabled), /p6_disposition/);

  const risks = campaign({ open_critical_risks_for_this_rc: 1 });
  assert.throws(() => buildPersonalRcDeclarationReport(risks), /open_critical_risks_for_this_rc/);

  const wrongGates = campaign({ target_gates: ["RC", "GMVP-LINUX"] });
  assert.throws(() => buildPersonalRcDeclarationReport(wrongGates), /target_gates/);

  const profileScope = campaign({ claim_scope: "profile" });
  assert.throws(() => buildPersonalRcDeclarationReport(profileScope), /claim_scope/);

  const missingNonClaim = campaign({
    explicit_non_claims: RC_REQUIRED_EXPLICIT_NON_CLAIMS.filter(
      (item) => item !== "does not claim Profile conformance",
    ),
  });
  assert.throws(
    () => buildPersonalRcDeclarationReport(missingNonClaim),
    /does not claim Profile conformance/,
  );
});

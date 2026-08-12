# 2026-08-12 — P7-T07 B01-W campaign preregistration (authoring only, not executed)

- Task: P7-T07 (Windows install surface) under
  `lease/personal/P7-T07/windows-install-surface`
- Gate: **B01-W** — Windows first-install/first-conversation qualification
- Campaign ID: `B01-W-clean-windows-first-install-first-conversation-001`
- Policy source: [ADR-0052 §3](../adr/0052-personal-windows-install-surface.md)
  mirroring the [ADR-0039](../adr/0039-personal-b01-six-attempt-campaign-policy.md)
  B01 successor rule
- Status: **preregistered, not started, not executable yet** (see §4
  prerequisites). This document creates no attempt, no denominator entry, no
  environment, and no claim.

## 1. Fixed campaign policy (bound now)

1. Fixed denominator: exactly **N=6** counted clean-reset outcomes; every
   started attempt is immutable and counted.
2. Threshold: at least **5 of 6** attempts succeed (>= 83.33% observed rate).
3. Critical safety failures must equal **zero**. A critical safety failure is
   any secret exposure (credential in argv/file/log/evidence), any install
   action outside the versioned install root and registered per-user task, or
   any unverified-artifact execution.
4. A complete aggregate report (all six outcomes, 95% binomial interval,
   successful-route median/p95 timings, every failure category) and an
   affirmative independent-verifier disposition are mandatory before B01-W
   can become `pass`.
5. Attempt journey (each on a snapshot-reset clean VM):
   render-bound bootstrap download → digest/signature-verified install into a
   versioned root → per-user least-privilege scheduled-task registration and
   daemon readiness on `127.0.0.1:48181` → graphical **hidden-input** Provider
   credential entry into the Windows Credential Manager backend → one bounded
   first conversation with a complete response → uninstall/cleanup with secret
   removal verified.
6. Redaction: no secret material, raw Provider traffic, or personal content in
   the ledger; timings, outcome classes, and digests only.
7. Claim ceiling: a completed passing campaign passes **B01-W only**. It does
   not create G1, GMVP-LINUX, Windows release, Profile, or containment claims.
   An incomplete or failed campaign must still produce a complete non-claim
   report.

## 2. Environment requirement (registered, not provisioned)

- Required environment: `B01-W-DESKTOP-001` — a dedicated clean Windows 11
  x86_64 (or Windows 10 22H2+) VM with PID-controlled snapshot reset, native
  per-user Credential Manager, a graphical interactive session for hidden
  input, system PowerShell 5.1 and System32 `curl.exe`, and **no** developer
  toolchain or preexisting CognitiveOS state.
- The environment must be registered and qualified in
  [PERSONAL-TEST-ENVIRONMENTS.md](../plan/PERSONAL-TEST-ENVIRONMENTS.md)
  before any attempt starts; qualification follows the registry §15 template
  (exact image, architecture, reset snapshot, collector version, operator and
  verifier identities).
- The sole active Linux B01 guest `B01-Desktop-Linux-002` and its host are not
  valid B01-W targets and are not touched by this campaign.

## 3. Execution-time bindings (bound at start, per §15 template)

Before attempt 1, the campaign ledger must bind: the exact pushed source
revision; the Windows bundle artifact, installer executable, manifest, SBOM,
and attestation digests; the trusted keyring version/key id; the exact Pi
package/version/SRI pins; the rendered bootstrap digest; the SecretStore
backend identity (`WindowsCredentialManagerStore`, ADR-0052 §1); the evidence
collector version and redaction procedure; and the operator plus independent
verifier identities.

## 4. Prerequisites that do not exist yet (owner/pipeline-gated)

1. **Windows release artifacts**: the release pipeline currently produces and
   signs Linux bundles only. A `cognitiveos-windows-x86_64.zip` artifact, a
   `cognitiveos-windows-bundle-installer.exe`, and their manifest/SBOM/
   attestation signing path must exist before any rendered bootstrap can be
   executed end to end.
2. **Provisioned campaign VM**: `B01-W-DESKTOP-001` must be created,
   snapshotted, and qualified by the owner/operator; no existing registered
   environment satisfies the requirement.
3. **Operator availability** for the graphical hidden-input credential entry
   and independent verifier disposition.

Until all three exist, B01-W remains authored-but-unexecutable; P7-T07 records
this honestly instead of substituting CI, fixture, WSL, or Linux evidence.

## 5. Non-claims

This preregistration makes no implementation, install, parity, Gate, release,
or Profile claim. It does not start the campaign, does not create an attempt
ledger, and does not modify any existing B01 (Linux) evidence.

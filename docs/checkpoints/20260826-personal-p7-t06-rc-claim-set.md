# P7-T06/D01 Personal Linux RC claim freeze

- Task: `P7-T06`
- Slice: `P7-T06/D01`
- Campaign: `PERSONAL-LINUX-RC-declaration/1`
- Binder: `tools/src/personal-rc-gate.mjs`
- Branch: `personal/P7-T06-rc-docs-support-matrix`
- Lease: `lease/personal/P7-T06/rc-docs-support-matrix`
- Date: 2026-08-26
- Classification: documentation + tooling freeze; not a Gate disposition
- Claim ceiling: `hypothesis`

This freeze owns the Personal Linux RC **claim set**. It does not set RC, Gate,
or Profile state, publish a GitHub Release, or run a new B01 guest campaign.

## 1. Claim scope

`claim_scope` is exactly `personal-linux-rc-declaration`.

The declaration may bind existing MVP Gate dispositions and operability
evidence. It must not:

- impersonate CognitiveOS Core Profile `implemented`;
- claim a production GitHub Release or production signing ceremony;
- claim Windows install parity (`B01-W`);
- enable Multi-Agent / B11;
- include B10/MCP/dynamic Tool or Web UI in the Linux RC product claim;
- promote B06/B07 observations to a benefit or Gate pass.

`implemented` for any MUST remains applicable-MUST evidence only (F-016).

## 2. P6 disposition

`p6_disposition` is `disabled-nogo` for **this RC**.

`P6-T01`..`P6-T04` stay `not-started`. This freeze does not cancel Phase 6; it
records that Multi-Agent is default-disabled and is not in the Linux RC claim.
NO-GO is the plan-legal result that unblocks RC (`PERSONAL-DEVELOPMENT-PLAN.md`
P6-T04 / P7-T06 acceptance).

## 3. Clean VM suite binding

The clean-VM suite for this RC is a **composition**, not a new B01 attempt and
not a mutation of `B01-Desktop-Linux-002`:

| Step | Bound evidence |
|---|---|
| install → init → provider → Pi → first conversation | B01 successor `002` (ADR-0039) closure |
| Task / Effect / recovery / unknown-outcome | Runtime Spine B02/B04/B05/B12 (ADR-0046) |
| Context correctness | B03 (ADR-0040) |
| Memory/Skill consumption | B08 (ADR-0048) |
| managed Pi + sidecar lifecycle | B09 (ADR-0047) |
| update / rollback / uninstall authority path | P7-T01 installer compensation + P7-T02 `plan_personal_lifecycle` (no public `cognitive uninstall`; no OS-level host uninstall claimed) |
| GMVP composition | ADR-0049 / P7-T08 |

A current-HEAD isolated install on `personal-linux-native-01` is **not** this
suite: the host user unit would collide, and the B01 guest is campaign-isolated.

## 4. Open critical risks for this RC

`open_critical_risks_for_this_rc` = **0**.

Carried Core findings that are **not** Personal RC blockers:

- `F-001` — evidence-growth P0; not a Personal product contract gap for this claim set.
- `F-015` — coverage growth P1; not a Personal RC blocker.

P7-T07 / `B01-W` remains a blocked Windows path and is an explicit non-claim,
not an open Linux RC critical risk.

## 5. Required binder observations

Evidence observations (each needs `true` plus a `sha256:` binding):

- Gate: `b01_mvp_pass`, `b02_mvp_pass`, `b03_mvp_pass`, `b04_mvp_pass`,
  `b05_mvp_pass`, `b08_mvp_pass`, `b09_mvp_pass`, `b12_mvp_pass`,
  `gmvp_linux_mvp_pass`
- Operability: `required_ci_both_platforms`, `six_resource_release_manifest`,
  `sbom_attestation_digest_bound`, `lifecycle_update_rollback_uninstall`,
  `support_matrix_matches_claim_set`, `runbooks_published`,
  `clean_vm_suite_bound`

Dispositions (must be `true`, meaning recorded):

- `p6_disabled_nogo`, `b06_b07_non_claim`, `b10_not_in_rc_claim`,
  `web_ui_non_blocking_not_in_rc`, `windows_no_install_parity`

Failure-first negatives live in `tools/test/personal-rc-gate.test.mjs`.

## 6. Non-claims

The binder and this freeze do not set Gate state, do not claim Profile, do not
claim production publication, and do not mutate an isolated campaign guest.

Composition report: [20260826-personal-p7-t06-rc-composition-report.md](20260826-personal-p7-t06-rc-composition-report.md).

# P7-T06/D03 Personal Linux RC composition report

- Task: `P7-T06`
- Slice: `P7-T06/D03`
- Campaign: `PERSONAL-LINUX-RC-declaration/1`
- Binder: `tools/src/personal-rc-gate.mjs`
- Claim freeze: [20260826-personal-p7-t06-rc-claim-set.md](20260826-personal-p7-t06-rc-claim-set.md)
- Date: 2026-08-26
- Classification: digest-bound composition; not a Gate disposition
- Claim ceiling: `hypothesis`

This report binds declared-scope B01–B12 MVP evidence, operability closures,
the D02 bilingual runbooks, and the support-matrix honesty freeze into one
Personal Linux RC declaration. The evaluator does not set RC, Gate, or Profile
state.

## Digests

- `suite_digest`: `sha256:7edaa50a8da7304b64195c2030d012bf501e3a19e879bf764b2a481a84036cf3`
  (raw SHA-256 of the D01 claim-set document)
- `trace_digest`: `sha256:cd36dd58f12cc9fe12853c66c1d4b01afde825a6765ce92011a0730e9fcb3e23`
  (raw SHA-256 of `docs/plan/PERSONAL-SUPPORT-MATRIX.md` after D01 honesty edits)
- `report_digest`: `sha256:2c36e4594c4318fa64bfd7017299b7cd858f1e3e33c8f57ae3d99d601acc62c3`
  (`CognitiveOS-Digest-V1` over `cognitiveos.personal.rc-declaration-report/0.1`)

`p6_disposition`: `disabled-nogo`. `open_critical_risks_for_this_rc`: `0`.
`claim_scope`: `personal-linux-rc-declaration`. `target_gates`: `["RC"]`.

## Gate observation bindings (raw file SHA-256)

| Observation | Bound file | Digest |
|---|---|---|
| `b01_mvp_pass` | [B01 six-attempt waiver and closure](20260809-personal-p1-t09-b01-six-attempt-waiver-and-closure.md) | `sha256:bed1f82e10e94692783a677e8385efe8adc0ed6f49c863aa2f8bc2bd34fd9396` |
| `b02_mvp_pass` | [Runtime Spine closure](20260811-personal-p2-t08-runtime-spine-closure.md) | `sha256:09fc5a6dd1639d967a2062df12658605d29ccc89b1f646302b54db5bebf1213f` |
| `b03_mvp_pass` | [B03 MVP closure](20260810-personal-p3-t06-b03-mvp-closure.md) | `sha256:6eb7117f13288fa35d7fd93a1553f61e12fec5c55ca1e2e0e41cb6c84eb576e6` |
| `b04_mvp_pass` | Runtime Spine closure (same as B02) | `sha256:09fc5a6dd1639d967a2062df12658605d29ccc89b1f646302b54db5bebf1213f` |
| `b05_mvp_pass` | Runtime Spine closure (same as B02) | `sha256:09fc5a6dd1639d967a2062df12658605d29ccc89b1f646302b54db5bebf1213f` |
| `b08_mvp_pass` | [B08 disposition](20260811-personal-p7-t08-d02-b08-disposition.md) | `sha256:1bbd215f7afac1334daa066a8088389395a844b31bc5f648a7a9c588e6bb868d` |
| `b09_mvp_pass` | [B09 managed Pi closure](20260811-personal-p5-t05-b09-managed-pi-closure.md) | `sha256:b9b76484f53e72f598db7fe647a2b660cec65e9e13cdba4f544dfaa8ee0b9892` |
| `b12_mvp_pass` | Runtime Spine closure (same as B02) | `sha256:09fc5a6dd1639d967a2062df12658605d29ccc89b1f646302b54db5bebf1213f` |
| `gmvp_linux_mvp_pass` | [GMVP D03 composition](20260811-personal-p7-t08-d03-gmvp-composition-evidence.md) | `sha256:9c95719b7ee71503a6e14aeca67839f16cae92f2bb1a09a72f8c3bd7d6c2d8ce` |

GMVP composition `report_digest` on that D03 file remains
`sha256:c7cbc45f78f8e214f69a2d1c42492175ac698504440aa05558528510445806d7`.

## Operability bindings (raw file SHA-256)

| Observation | Bound file | Digest |
|---|---|---|
| `required_ci_both_platforms` | [P7-T05 closure](20260826-personal-p7-t05-control-plane-redesign-closure.md) (required CI [32942980183](https://github.com/agentkernel/cognitive-os/actions/runs/32942980183) SUCCESS at `b147711a`) | `sha256:43751af17c1b3124fd595040ea3cc9784794675ad3edb604e706498e1594b007` |
| `six_resource_release_manifest` | [P7-T01 release pipeline](20260811-personal-p7-t01-release-pipeline-closure.md) | `sha256:88cf86f6fd33f72433715cff317b068877410b9b430741eb773f87b2ee18388e` |
| `sbom_attestation_digest_bound` | P7-T01 release pipeline (same file) | `sha256:88cf86f6fd33f72433715cff317b068877410b9b430741eb773f87b2ee18388e` |
| `lifecycle_update_rollback_uninstall` | [P7-T02 lifecycle](20260811-personal-p7-t02-lifecycle-backup-closure.md) | `sha256:64798ecd0cf8b6a02b9b8d91ac69dbd119d0c6fc92b3a5b6171c4acc5abb1253` |
| `support_matrix_matches_claim_set` | `docs/plan/PERSONAL-SUPPORT-MATRIX.md` | `sha256:cd36dd58f12cc9fe12853c66c1d4b01afde825a6765ce92011a0730e9fcb3e23` |
| `runbooks_published` | `personal/handbook/en/user/rc-and-support.md` (zh-CN twin paired) | `sha256:e30f261fd53cfdc24c6414026e49bc5ceaeb373afa055aa0a4bca51284c6eae4` |
| `clean_vm_suite_bound` | B01 successor `002` six-attempt waiver and closure (same digest as `b01_mvp_pass`); update/rollback/uninstall are the P7-T01/P7-T02 authority path, not a new guest campaign | `sha256:bed1f82e10e94692783a677e8385efe8adc0ed6f49c863aa2f8bc2bd34fd9396` |

This PR's own required Ubuntu/Windows CI is recorded at D04. The parent-main CI
binding above proves both-platform required CI exists for the product line; it
is not this task's merge check.

## Dispositions (true = recorded)

- `p6_disabled_nogo`
- `b06_b07_non_claim`
- `b10_not_in_rc_claim`
- `web_ui_non_blocking_not_in_rc`
- `windows_no_install_parity`

## Non-claims

- does not set Gate state
- does not claim Profile conformance
- does not claim a production GitHub Release or production signing ceremony
- does not claim Windows install parity (B01-W)
- does not enable Multi-Agent / B11
- does not include B10/MCP/dynamic Tool in the Linux RC claim
- does not include Web UI in the Linux RC claim
- does not promote B06/B07 observations to a benefit or Gate pass

## Harness validation

- Local `node --test tools/test/personal-rc-gate.test.mjs` **2/2 pass**
- `pnpm -C tools test` includes `personal-rc-gate`

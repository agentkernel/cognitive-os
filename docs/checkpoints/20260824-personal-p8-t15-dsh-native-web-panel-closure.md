# P8-T15 native dsh Web UI control panel — closure

- Task: `P8-T15` / slices `P8-T15/D01`–`D04`
- Status: `done`
- Branch: `personal/P8-T15-dsh-native-web-panel` (deleted after merge)
- PR: [#265](https://github.com/agentkernel/cognitive-os/pull/265) **merged** at `main@562d2a5d`
- Content head: `f846540f`
- Lease: closed `lease/personal/P8-T15/dsh-native-web-panel`
- Change class: implementation + handbook + plan closure
- Claim ceiling: `hypothesis`
- Non-claims: no Gate, release, Profile, B01, EVAL, Personal `/ui/` panel, or Agent-benefit promotion

## Acceptance mapping

| Formal acceptance | Evidence |
|---|---|
| `cognitive dsh web` loopback-only native panel; dist fail-closed; Path B preserved | D01 admin-cli **9/9**, Node preflight **5/5**, negatives (`0.0.0.0`, missing dist) |
| linux-002 `:3080` GET `/` native SPA HTML; status ACTIVE | D02 Cos-installed panel **pass**; Path B `--print` retained |
| Bilingual operator handbook vs Personal `/ui/` | D03 authored + generated pages; docs-sync fingerprints refreshed in PR #265 |
| Apply Cos dsh binding to running web; Models follow bound catalog | D04 daemon overlay + helper reload; `p8_t13` **7/7** including catalog sync test; guest Cos LongCat-only settings |
| Provider create accepts OpenAI-compatible chat URL pastes | `endpoint_trust` **8/8** + control-plane create test at `8b9d09cb` |
| Draft PR → required CI → merge | PR #265 ready; required CI run `32687964519` at `f846540f` passed Ubuntu, Windows, required-ci |

## Validation

| Unit | Environment | Revision | Result |
|---|---|---|---|
| resolve validation route | GitHub Actions | `f846540f` | **pass** |
| verify (ubuntu-latest) | GitHub Actions | `f846540f` | **pass** (3m23s) |
| verify (windows-latest) | GitHub Actions | `f846540f` | **pass** (11m41s) |
| required-ci | GitHub Actions | `f846540f` | **pass** (run `32687964519`) |
| `p8_t13_provider_control_plane` | `DEV-LINUX-NATIVE-01` | `4bfaee66` | **pass** 7/7 |
| `p8_t11_dsh_runtime` | `DEV-LINUX-NATIVE-01` | `4bfaee66` | **pass** 1/1 |
| `dsh-web-preflight.test.mjs` | local Node | `4bfaee66` | **pass** 5/5 |
| Clippy `-D warnings` kernel-server+admin-cli | `DEV-LINUX-NATIVE-01` | `4bfaee66` | **pass** |
| Guest daemon + Cos web | linux-002 | `4bfaee66` → `f846540f` | **pass** pre-merge guest **517492** / **517786**; post-closure replace to merged `main` revision |
| Clients Apply copy | cognitiveos-clients | PR #4 @ `0320c1a` | **not merged** (separate repo) |
| Local Windows GNU Rust | `DEV-WIN-GNU-01` | — | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |

## Unique next action

Wait for a fresh owner delivery instruction. Do not auto-claim P6 / P7-T06 / P7-T07.

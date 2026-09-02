# DOC-P13-DRIFT-FIX — running report (documentation drift alignment)

- Activity: owner-directed documentation delivery `DOC-P13-DRIFT-FIX`
  (PERSONAL-DEVELOPMENT-PLAN Phase 13「配套维护交付」row; prerequisite of `P0-T09`)
- Lease: `lease/personal/DOC-P13-DRIFT-FIX/build-order-and-pi-package`
- Branch: `personal/DOC-P13-DRIFT-FIX` (worktree `D:\agent-kernel-wt-doc-p13-drift`, base `origin/main@a0465653`)
- Environment for every local unit: `DEV-WIN-GNU-01` (Windows PowerShell 5.1; Node tooling only; no Rust link)
- Claim ceiling: `hypothesis`. Documentation/static-consistency evidence only — no Gate,
  release, Profile, T15, or Windows-support claim. `not-run` is never pass.
- Reporting rule: `TEST-REPORT-INCREMENTAL-01` — each unit appended on completion; append-only.

## 1. Authority check for item (b) — Pi package name

| Source | Value | Role |
|---|---|---|
| `personal/crates/cognitive-runtime/src/installer.rs` `OFFICIAL_PI_PACKAGE` | `@earendil-works/pi-coding-agent` (`OFFICIAL_PI_VERSION = "0.81.1"`) | code constant (authority) |
| `docs/plan/PERSONAL-TEST-ENVIRONMENTS.md` §1 "Pi" pin | `@earendil-works/pi-coding-agent@0.81.1` | environment registry |
| `docs/plan/plan.md` PI-02 | `@earendil-works/pi-coding-agent` | research/pin card |
| `personal/packages/pi-cognitiveos/src/pin.ts`, `personal/apps/pi-agent-adapter/fixtures/p0_t06_extension.ts`, `personal/crates/cognitive-store/tests/installation_store.rs` | `@earendil-works/pi-coding-agent` | code/tests |
| handbook `reference/compatibility.md` (en, zh-CN), `developer/agent-and-pi-lifecycle.md` (en, zh-CN) **before this delivery** | `@mariozechner/pi` | drifted derived layer |

Disposition: code, environment registry and PI-02 agree; the handbook is the drifted
side. Unify the four handbook pages to the code constant. **No owner decision needed**
(the "handbook is authority" stop condition did not trigger).

## 2. Edge-set comparison for item (a) — Phase 13 build order

Formal plan: `docs/plan/PERSONAL-DEVELOPMENT-PLAN.md` "Phase 13 - Personal 2.0.0 completion"
建造顺序 mermaid (authoritative). Index: `personal/docs/architecture/personal-2.0.0-dev-prep-index.md`
"Phase 13 build order" mermaid. Node ids normalized by stripping the `P13`/`P11` prefix
(`P13T12a` ≡ `T12a`, `P11T15` ≡ `T15`). `-->` solid, `-.->` dashed.

| # | Edge (normalized) | Kind | Formal plan | Index before | Index after |
|---|---|---|---|---|---|
| 1 | T01→T02 | solid | yes | yes | yes |
| 2 | T01→T03 | solid | yes | yes | yes |
| 3 | T01→T12a | solid | yes | yes | yes |
| 4 | T02→T04 | solid | yes | yes | yes |
| 5 | T02→T05 | solid | yes | yes | yes |
| 6 | T02→T06 | solid | yes | yes | yes |
| 7 | T03→T06 | solid | yes | yes | yes |
| 8 | T03→T10 | solid | yes | yes | yes |
| 9 | T04→T10 | solid | yes | yes | yes |
| 10 | T04→T11 | solid | yes | yes | yes |
| 11 | T05→T11 | solid | yes | yes | yes |
| 12 | T06→T07 | dashed | yes | yes | yes |
| 13 | T02→T08 | dashed | yes | yes | yes |
| 14 | T05→T09 | dashed | yes | yes | yes |
| 15 | T04→T12b | solid | yes | yes | yes |
| 16 | **T05→T12b** | solid | yes | **missing** | **added** |
| 17 | **T07→T12b** | solid | yes | **missing** | **added** |
| 18 | T08→T12b | solid | yes | yes | yes |
| 19 | T12a→T12b | solid | yes | yes | yes |
| 20 | T02→T13 | solid | yes | yes | yes |
| 21 | **T05→T13** | solid | yes | **missing** | **added** |
| 22 | T08→T13 | solid | yes | yes | yes |
| 23 | T12b→T15 | solid | yes | yes | yes |
| 24 | T13→T15 | solid | yes | yes | yes |
| 25 | **T09→T15** | solid | yes | **missing** | **added** |
| 26 | **T10→T15** | solid | yes | **missing** | **added** |
| 27 | **T11→T15** | solid | yes | **missing** | **added** |

Totals: formal plan 27 edges (24 solid, 3 dashed); index before 21 edges (0 extra edges,
6 missing); index after 27 edges — equal to the formal plan edge-for-edge, including
dashed/solid kind. The formal plan was **not** modified. Node label text differs between
the two graphs by design (index uses `P13_*` labels); only ids/edges are compared.

## 3. Units

| # | Unit | Instrument | Environment | Revision | Result | Notes |
|---|---|---|---|---|---|---|
| U1 | (a) dev-prep index edge alignment | manual edit + edge table §2 | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | 6 edges added; 27/27 equal |
| U2 | (b) authority confirmation | grep of code/registry/plan (table §1) | `DEV-WIN-GNU-01` | `a0465653` | **pass** | code constant is authority; no owner stop |
| U3 | (b) 4 handbook pages unified to `@earendil-works/pi-coding-agent` | manual edit | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | repo grep for `@mariozechner/pi` outside the plan row = 0 after edit (verified in U9) |
| U4 | (c) autocrlf note en + zh-CN | manual edit; `git config core.autocrlf` = `true`; `.gitattributes` line 4 = `* text=auto eol=lf` | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | note states no local config change is needed |
| U5 | `pnpm run check:consistency` | `tools/src/check-consistency.mjs` (DOC allowlist extended to exact `docs/checkpoints/` files) | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | `check-consistency: OK` (275 req, 55 error codes, 74 schemas, 89 vectors); new lease row accepted |
| U6 | `pnpm run check:handbook` (first run) | `tools/src/check-handbook.mjs` | `DEV-WIN-GNU-01` | worktree (pre-commit) | **fail → fixed** | HB008 fingerprint drift on `dev.architecture-overview` (source: dev-prep index) and `dev.conformance-testing` (source: `check-consistency.mjs`) in both locales; pages reviewed (index mirrors formal plan; DOC lease class documented) and `fill-handbook-fingerprints` refreshed 4 pages |
| U7 | `pnpm run check:handbook` (rerun) + `node tools/src/generate-handbook.mjs --check` | handbook checker + generator | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | `check-handbook: OK (58 documents x 2 locales, 9 generated)`; `generate-handbook --check: OK (18 pages byte-identical)` |
| U8 | `pnpm run check:rules` | `tools/src/check-agent-rules.mjs` | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | `OK (4 rules, 0 commands, 88 path references, 5 local-only warning(s))` — the worktree has no untracked local editor assets, so the warnings are the expected clean-checkout class |
| U9 | `git diff --check` + stale-name grep | git; `rg "@mariozechner/pi"` | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | no whitespace errors; remaining mentions of the old name are only the plan row describing the drift and §1 of this report |
| U10 | repo-tools build + tests | `pnpm --filter @cognitiveos/repo-tools run build`; `node --test test/*.test.mjs` | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | build OK; **115/115** tests pass, including the updated DOC-lease fixtures (checkpoint file accepted; bare `docs/checkpoints/` directory rejected) |
| U11 | Rust build/test/Clippy | — | `DEV-WIN-GNU-01` | — | **not-run** | documentation-only change; no Rust surface touched; `RUST-LINK-DEV-WIN-GNU-01` applies |
| U12 | commit + push + Draft PR | docs-sync-gate pre-commit hook; `git push`; `gh pr create --draft` | `DEV-WIN-GNU-01` | `12e84b7c` | **pass** | hook OK; push succeeded after a ~40 min GitHub TLS outage on this host (schannel handshake failures direct and via the local proxy; retried with backoff, no config changed); Draft PR [#309](https://github.com/agentkernel/cognitive-os/pull/309) |
| U13 | required CI | `CI-UBUNTU-01` + `CI-WINDOWS-MSVC-01`, run [33670770754](https://github.com/agentkernel/cognitive-os/actions/runs/33670770754) | GitHub Actions | `12e84b7c` | **pass** | resolve 3s; verify (ubuntu-latest) 3m30s; verify (windows-latest) 10m7s; required-ci 3s — all SUCCESS |

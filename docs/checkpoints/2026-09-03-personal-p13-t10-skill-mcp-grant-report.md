# P13-T10 Skill/MCP security-reviewed acquire + scoped grant — running report

- Task: `P13-T10` / slice `P13-T10/D01`
- Change class: `implementation-only` (reuses v27 `p11_install_fact` / `p11_grant` and v30 `grant-expansion`; no new migration; no `core/specs`; no marketplace / engine store / second grant table)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P13-T10/skill-mcp-grant`
- Branch: `personal/P13-T10-skill-mcp-grant` (worktree `D:\agent-kernel-wt-P13-T10`; original `d:\agent-kernel` not used for writes)
- Claim ceiling: `hypothesis` (A7: local / CI / Linux native is not Gate / release / Profile; supply-chain host E2E stays `not-run` until `P13-T13`)
- Evaluation routing: **OFF**

## Unique next action

Required CI + `DEV-LINUX-NATIVE-01` focused store + kernel-server + clippy on Draft PR [#318](https://github.com/agentkernel/cognitive-os/pull/318) HEAD `0d6fc2af`. Do not merge until D01 acceptance. Supply-chain host E2E remains `not-run`.

## Failure-first (D01)

| ID | Negative | Surface |
|---|---|---|
| N1 | install ≠ grant: install-phase confirm writes only an InstallFact and consumes the preview (`granted: false`); catalog stays empty until a grant-phase confirm | store + HTTP |
| N2 | unreviewed install refused: incomplete `SecurityReview` never records an InstallFact | store + Dual Track |
| N3 | hidden-instruction / prompt-injection must be `none`; otherwise refuse | store |
| N4 | ambient / marketplace / engine-store / generic `resource:` sources refused; sources must be `https://` or `owner://` | store + HTTP |
| N5 | chat / task-channel aliases 403; chat cannot Approve | HTTP + Dual Track |
| N6 | ambient grant refused; grant-phase preview requires an InstallFact | store |
| N7 | no fake Activate / Install / Confirm on member-config chrome | Dual Track |

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

Units are appended **immediately** after each finishes. `not-run` is never pass.

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-03 | Resumed worktree `D:\agent-kernel-wt-P13-T10` on `personal/P13-T10-skill-mcp-grant` from `origin/main`; lease already claimed | recorded | docs-only | worktree | `d:\agent-kernel` not used for writes; T06/T07/T08 isolation held |
| 2026-09-03 | Failure-first store tests written first (`p13_t10_capability_acquisition.rs`); observed compile-fail (APIs missing), then implemented `SecurityReview` / `record_reviewed_install_fact` / `admit_discovery` / `compat_test` / `review_update` / `rollback_install` | recorded | `DEV-WIN-GNU-01` then MSVC override | worktree | no new migration |
| 2026-09-03 | `cargo test -p cognitive-store --test p13_t10_capability_acquisition` | **pass** 6/6 | local MSVC override (`CARGO_PROFILE_DEV_DEBUG=0`) | worktree | development evidence only |
| 2026-09-03 | `cargo test -p kernel-server http_capability_acquire_install_is_not_grant_and_refusals` | **pass** | local MSVC override | worktree | install ≠ grant; refusals; catalog lists installs with `install_is_not_grant` |
| 2026-09-03 | Dual Track: `capabilityAcquire.ts` + member-config acquire panel + HITL grant-expansion review line; no Activate | recorded | `DEV-WIN-GNU-01` (Node) | worktree | acquire panel is a sibling of the catalog `DaemonReadPanel` so empty grants still show Request acquire preview |
| 2026-09-03 | `cargo test -p cognitive-store --test p13_t10_capability_acquisition --locked` | **pass** 6/6 | local MSVC override (`CARGO_PROFILE_DEV_DEBUG=0`) | worktree | development evidence only |
| 2026-09-03 | `cargo test -p kernel-server http_capability_acquire_install_is_not_grant_and_refusals --locked` | **pass** 1/1 | local MSVC override | worktree | syntax fix in acquire match arm; install ≠ grant |
| 2026-09-03 | `clients/pc/web` focused vitest (memberConfig / capabilityAcquire / hitl / hitlConfirm) | **pass** 23/23 (4 files) | `DEV-WIN-GNU-01` (Node) | worktree | empty catalog still shows acquire; no Activate |
| 2026-09-03 | `cargo fmt --all -- --check` / `generate-handbook` + `fill-handbook-fingerprints` / `check-handbook` / `generate-handbook --check` | **pass** | `DEV-WIN-GNU-01` + MSVC fmt | worktree | 58×2 handbook; 18 generated pages byte-identical |
| 2026-09-03 | D01 checkpoint pushed; Draft PR [#318](https://github.com/agentkernel/cognitive-os/pull/318) opened | recorded | GitHub | `0d6fc2af` | Draft until full acceptance |
| 2026-09-03 | Required CI / `DEV-LINUX-NATIVE-01` | **not-run** | routed | `0d6fc2af` | exact-revision after Draft PR |

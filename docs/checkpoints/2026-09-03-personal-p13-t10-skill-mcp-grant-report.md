# P13-T10 Skill/MCP security-reviewed acquire + scoped grant — running report

- Task: `P13-T10` / slice `P13-T10/D01`
- Change class: `implementation-only` (reuses v27 `p11_install_fact` / `p11_grant` and v30 `grant-expansion`; no new migration; no `core/specs`; no marketplace / engine store / second grant table)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P13-T10/skill-mcp-grant`
- Branch: `personal/P13-T10-skill-mcp-grant` (worktree `D:\agent-kernel-wt-P13-T10`; original `d:\agent-kernel` not used for writes)
- Claim ceiling: `hypothesis` (A7: local / CI / Linux native is not Gate / release / Profile; supply-chain host E2E stays `not-run` until `P13-T13`)
- Evaluation routing: **OFF**

## Unique next action

**D01 accepted (2026-09-03).** Install ≠ grant; chrome is **Request acquire preview** only; there is **no Activate**. Folded `origin/main` after T06 close (PR #316 at `main@23355afb`). Lease closed this delivery. Next: required CI green on this folded HEAD → `gh pr ready 318` → merge. Then claim **`P13-T09`** if still unclaimed. Do not claim sibling-owned `P13-T07` / `T08`. Supply-chain host E2E remains `not-run` until P13-T13.

## Formal D01 acceptance map (plan card + Delivery Slice `P13-T10/D01`)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| 助手主导发现（带 sources） | store `admit_discovery` + HTTP `POST capability.discover`; sources must be `https://` or `owner://` | N4 marketplace / engine-store / ambient / generic `resource:` refused | store `marketplace_engine_store_and_unpinned_sources_are_refused`; HTTP acquire refusals |
| 结构化安全评审 | `SecurityReview` (source / license / hidden-instruction / prompt-injection / file·network·command intent; MCP: dependencies / executable / secret / tool perms / supply-chain) required before InstallFact | N2 incomplete review; N3 hidden / injection ≠ `none` | store `unreviewed_install_is_refused`, `injection_or_hidden_instruction_refuses_install`; HTTP 422 |
| 首次安装/扩权前 exact Owner 画布 preview | `capability.acquire` mints a v30 `grant-expansion` preview (`granted: false`); member-config button is **Request acquire preview** and deep-links the canvas; chat cannot Confirm | N5 chat/task 403; N7 no Activate / Install / Confirm chrome | Dual Track `memberConfig.test.tsx` (Request acquire preview; `fakeActionLabels` empty); HTTP `chat_can_confirm: false` |
| 版本锁定 capability artifact | InstallFact carries `version_pin`; `compat_test` / `review_update` / `rollback_install` never silent-grant | rollback refuses the rolled-back pin; catalog stays empty | store `update_review_compat_and_rollback_do_not_silent_grant` |
| 独立 Project/Member grant | grant-phase preview requires an InstallFact; grant-phase confirm writes `p11_grant` with scope; install-phase confirm writes InstallFact only | N1 install ≠ grant; N6 ambient grant refused | store `reviewed_install_is_not_a_grant` (catalog empty; `invoke_tool` refused); HTTP confirm `"granted":false` and catalog has no `"catalog":["mcp:search"]` |
| 更新评审 / 兼容测试 / 回滚 | HTTP `capability.compat-test` / `.rollback`; store `review_update` | never silent-grant | Linux store 6/6 at `d861d341` |
| 关闭门 sentence: 发现 → 评审 → exact Owner 画布 preview → 版本锁定 → 按范围 grant → 更新/兼容/回滚 | vertical path above; Dual Track still uses **Request acquire preview** only | 安装即授权; 未评审自动安装; 聊天 Approve; ambient grant; engine store / marketplace; grant 无范围 | required CI [33747031610](https://github.com/agentkernel/cognitive-os/actions/runs/33747031610) **SUCCESS** @ `d861d341`; Linux store 6/6 + HTTP 1/1 + clippy |

Honesty: implementation + supported validation are green at `d861d341`. Fold/acceptance commits after that SHA are documentation / sibling-ledger only. Supply-chain host E2E stays `not-run` until `P13-T13`. No Activate was invented.

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
| 2026-09-03 | Dual Track focused vitest first run | **fail** 2/23 | `DEV-WIN-GNU-01` (Node) | worktree | acquire panel was inside `DaemonReadPanel`; empty catalog returned only EmptyState |
| 2026-09-03 | Dual Track focused vitest after sibling-panel fix | **pass** 23/23 | `DEV-WIN-GNU-01` (Node) | later committed in `0d6fc2af` | panel is a sibling of the catalog panel |
| 2026-09-03 | Required CI [33745159652](https://github.com/agentkernel/cognitive-os/actions/runs/33745159652) | **cancelled** | `CI-UBUNTU-01` | `0d6fc2af` | superseded by persist push; not required-CI green |
| 2026-09-03 | Required CI [33745410565](https://github.com/agentkernel/cognitive-os/actions/runs/33745410565) ubuntu Clippy | **fail** | `CI-UBUNTU-01` | `5a1d33e8` | `clippy::collapsible_if` in `confirm` install-phase guards; rust tests had passed |
| 2026-09-03 | Collapse install-phase confirm `if` chain; local `cargo clippy -p kernel-server --all-targets --locked -- -D warnings` | **pass** | local MSVC override | worktree | development evidence only |
| 2026-09-03 | `DEV-LINUX-NATIVE-01` GitHub fetch of `5a1d33e8` | **fail** | `wuz@192.168.1.2` | `5a1d33e8` | `fatal: pack has 28 unresolved deltas`; bundle+SCP after the Clippy repair push |
| 2026-09-03 | `DEV-LINUX-NATIVE-01` exact `d861d341` (`/home/wuz/cognitiveos-personal-worktrees/p13-t10-d861d341`, bundle fetch, `DIRTY=0`) store `p13_t10_capability_acquisition` | **pass** 6/6 | `DEV-LINUX-NATIVE-01` | `d861d341` | install ≠ grant; injection/hidden; marketplace/engine-store; unreviewed; update/compat/rollback; chat/task cannot install |
| 2026-09-03 | `DEV-LINUX-NATIVE-01` `cargo test -p kernel-server http_capability_acquire` | **pass** 1/1 | `DEV-LINUX-NATIVE-01` | `d861d341` | `http_capability_acquire_install_is_not_grant_and_refusals` |
| 2026-09-03 | `DEV-LINUX-NATIVE-01` `cargo clippy -p cognitive-store -p kernel-server --all-targets --locked -- -D warnings` | **pass** | `DEV-LINUX-NATIVE-01` | `d861d341` | focused crates; not a workspace substitute for required CI |
| 2026-09-03 | Required CI [33747031610](https://github.com/agentkernel/cognitive-os/actions/runs/33747031610) `verify (ubuntu-latest)` | **pass** | `CI-UBUNTU-01` | `d861d341` | Clippy included; rustfmt / consistency / handbook / conformance all success |
| 2026-09-03 | Required CI [33747031610](https://github.com/agentkernel/cognitive-os/actions/runs/33747031610) `verify (windows-latest)` | **pass** | `CI-WINDOWS-MSVC-01` | `d861d341` | 11m53s |
| 2026-09-03 | Required CI [33747031610](https://github.com/agentkernel/cognitive-os/actions/runs/33747031610) `required-ci` | **pass** | GitHub | `d861d341` | all required checks green |
| 2026-09-03 | `DEV-LINUX-NATIVE-01` bundle+SCP clone @ `d861d341` (worktree `/home/wuz/agent-kernel-worktrees/p13-t10-d861d341`) | recorded | `wuz@192.168.1.2` | `d861d341` | GitHub fetch broken on host; bundle route used |
| 2026-09-03 | Owner pause | recorded | — | `d861d341` | finish in-flight CI/Linux only; no merge; no new card |
| 2026-09-03 | Pause-docs commit `83c603b4` | recorded | GitHub | `83c603b4` | pushed; untracked `p13-t10-d861d341.bundle` left unstaged (A8) |
| 2026-09-03 | Owner lifts pause for T10 closure; formal D01 acceptance map | recorded | docs | fold HEAD | install ≠ grant; Request acquire preview only; no Activate; no product gap found |
| 2026-09-03 | Fold `origin/main` (T06 done PR #316 at `main@23355afb`) | recorded | worktree | fold HEAD | PROGRESS/plan T10 rows kept; T06 done facts from main; handbook fingerprints to regenerate |

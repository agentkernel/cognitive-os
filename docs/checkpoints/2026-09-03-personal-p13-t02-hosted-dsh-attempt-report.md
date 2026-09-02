# P13-T02 Hosted DSH real Attempt loop — running report

- Task: `P13-T02` / slices `P13-T02/D01` → `P13-T02/D02`
- Change class: `implementation-only` (daemon-owned stdio broker + real child spawn in `cognitive-runtime`; v36 Attempt / frame / artifact-fact ledger in `cognitive-store`; management HTTP `dsh.hosted.attempt.*` routes; minimal `runs` read; no `core/specs`, no Lane-CTR, no new first-level chrome, no Pi Member engine)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P13-T02/hosted-dsh-attempt`
- Branch: `personal/P13-T02-hosted-dsh-attempt` (worktree `D:\agent-kernel-wt-p13-t02`; original `d:\agent-kernel` untouched, A8 protected)
- Base: `origin/main@a04656531b280d75ce4058022466fb0234c07083`
- Claim ceiling: `hypothesis` (A7: local / CI / Linux native is not Gate / release / Profile; Linux real spawn closes "implementation exists" only; Windows sandbox / ACL / supply-chain cells stay `not-run` until `P13-T13`)
- Evaluation routing: **OFF**

## Unique next action

Write failure-first negatives (broker / store / HTTP), observe them fail on a supported route, implement, checkpoint, then run required CI + exact-revision `DEV-LINUX-NATIVE-01` real child spawn.

## Identifier

Hidden engine pin `cognitiveos.personal.hidden-hosted-dsh/0.1`; exact artifact `528c682e061696f5a160f363f236ecbf53cbd006` (= `DSH_PACKAGE_REVISION`); Path B proxy `POST /provider/v1/dsh/chat/completions` remains the only secret-bearing path. Reused, not rebuilt: P11-T07 `p11_hosted_dsh_child` identity + `runtime_binding_ref`; P2-T06 supervisor seam shape (PID ownership / timeout / orphan); P11-T14 persist-before-dispatch ledger shape.

## Failure-first (D01)

| ID | Negative | Surface |
|---|---|---|
| N1 | process death ≠ completion (exit 0, non-zero, signal/kill, timeout all land as `terminal_kind` ≠ success; `completion_claimed = 0`; `verification_status = not-run`) | runtime broker + store attempt ledger |
| N2 | unknown / free-text / `ok` / `{"status":"success"}` child output ≠ success (unknown lines counted, never promoted) | runtime broker + store |
| N3 | secret never in child env / argv (secret-shaped key/value, `sk-`, bearer, `ssv1:` refused before spawn) | runtime broker pre-spawn checks |
| N4 | child direct Provider refused (Provider host / base-URL / `--api-key-file` / `--provider-path a` in launch plan refused; child `provider_request` / non-loopback URL frames rejected and recorded) | runtime broker |
| N5 | native MCP / base tool / HMR / home patch refused | runtime broker pre-spawn checks |
| N6 | bounded Context payload (oversize refused; digest bound into request frame) | runtime broker |
| N7 | heartbeat frames never write authority; observation frames never advance Task state | store attempt ledger (frames are observations) |
| N8 | task channel cannot run / observe attempts (403); artifact digest mismatch refused (422); GNU fence recorded | kernel-server routes |
| N9 | persist-before-dispatch: attempt row exists before spawn; crash-shaped `persisted`-not-dispatched row reconciles to `unknown-outcome`, never success | store |

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

Units are appended **immediately** after each finishes. `not-run` is never pass.

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-03 | Lease claim + PROGRESS / formal plan / plan.md status → `P13-T02` in-progress, `P13-T02/D01` in-progress | recorded | docs-only (`DEV-WIN-GNU-01`) | worktree, uncommitted | `git fetch origin` failed twice (local proxy TLS handshake, `github.com` → fake-ip `198.18.0.10`); worktree created from local `origin/main@a0465653` (== local `main`); will re-fetch before first push |
| 2026-09-03 | Resume after aborted worker: inherited edits reviewed (broker, lease row, plan/PROGRESS claim); lease writable paths trimmed to the real change set; `git fetch origin` / `gh auth status` / `ssh git@github.com` all fail on this host (proxy fake-ip `198.18.0.10`) | recorded | `DEV-WIN-GNU-01` | worktree, uncommitted | Network fault is recoverable, not a blocker: implementation continues; push/PR retried later (fallback: exact-revision bundle pushed from `DEV-LINUX-NATIVE-01`, which reaches GitHub) |
| 2026-09-03 | D01 failure-first tests written: store `p13_t02_hosted_dsh_attempt.rs` (9 tests: artifact facts health/update/rollback, unhealthy refuses spawn, persist-before-dispatch → terminal, process death ≠ completion incl. schema CHECKs, unknown output ≠ success, frames never write authority, crash-shaped rows → unknown-outcome, bounded/secret-free context, task channel/unseated/Pi refused); runtime `p13_t02_hosted_dsh_broker.rs` (7 tests with **real `node` children**: stdin Context + allowlisted env, exit 0/7/timeout-kill/spawn-failed ≠ completion, unknown output, direct Provider refused pre-spawn and in-stream, secret env/argv/native escape never spawn (marker file), bounded stdout, artifact observe/resolve health classes) | recorded | `DEV-WIN-GNU-01` | worktree, uncommitted | `cargo test` is `not-run` here (`RUST-LINK-DEV-WIN-GNU-01`); first execution is CI + `DEV-LINUX-NATIVE-01` |
| 2026-09-03 | `node --test scripts/hosted-attempt-child.test.mjs` (product child script; fake pinned dsh CLI + fake loopback daemon) | **pass** 12/12 | `DEV-WIN-GNU-01` (Node 22) | worktree, uncommitted | Refusals: Path A / `--api-key-file` / `--mcp` / `--direct-base-url` exit 2 before spawn; non-request frame exit 2; `--revision` ≠ pin exit 3; pin-file drift exit 3; dsh CLI missing exit 4 (`dsh-cli-missing`, response `failed`); non-loopback origin / absent bootstrap / closed port exit 5. Full run: `child.started` (context digest) → `dsh.cli.compiled-lib` → `provider.proxy.bound` (model from `agent-bindings`) → `dsh.spawned` → observations → one `DeliverableDraft` candidate → `response done`, exit 0, bearer/bootstrap never on stdout, patch `baseURL` = loopback `/provider/v1/dsh`, dsh env has no secret-shaped keys. Slow dsh killed at budget → `failed / timed-out` exit 6; dsh exit 2 → `failed / dsh-exit-2`, text stays a candidate. First attempt hung: `spawnSync` blocked the fake daemon's event loop — test harness fixed to async `spawn` (test defect, not product) |

| 2026-09-03 | D01 + D02 implementation written: store v36 `hosted_dsh_attempt.rs` (artifact facts, Attempt Intent → dispatched → terminal, frames, unknown-outcome reconcile), `hosted_dsh.rs` `observe_spawn`, runtime broker (`env_clear` + allowlist, stdin request frame with `timeout_ms`, Unix process-group kill, `HostedDshArtifact::observe/resolve` + child-script digest, `ledger_frames`), kernel-server `hosted_dsh_attempt.rs` routes (`attempt.run/list/detail`, `artifact.check/facts`; task aliases 403) + startup reconcile in `serve_personal_loopback`, product child `hosted-attempt-child.mjs` | recorded | `DEV-WIN-GNU-01` | worktree, uncommitted | vertical: management HTTP → store Intent → v31 child identity → real spawn → observations → daemon terminal |
| 2026-09-03 | `cargo fmt --all -- --check` | **pass** | `DEV-WIN-GNU-01` | worktree, uncommitted | after `cargo fmt --all`; formatting only, no link |
| 2026-09-03 | `cargo build/test/clippy` (store `p13_t02_hosted_dsh_attempt`, runtime `p13_t02_hosted_dsh_broker`, kernel-server `hosted_dsh_attempt::tests`) | **not-run** | `DEV-WIN-GNU-01` | worktree, uncommitted | `RUST-LINK-DEV-WIN-GNU-01`; routed to required CI + exact-revision `DEV-LINUX-NATIVE-01` |
| 2026-09-03 | `node tools/src/generate-handbook.mjs` (10 new `dsh.hosted.attempt.*` / `dsh.hosted.artifact.*` route annotations; `hosted_dsh_attempt.rs` added to definition sources) + `fill-handbook-fingerprints` (22 pages) | **pass** | `DEV-WIN-GNU-01` (Node) | worktree, uncommitted | `http-api` both locales regenerated; store-migrations (v36), daemon-and-http, agent-and-pi-lifecycle, capability-status hand-updated in en + zh-CN |
| 2026-09-03 | `node tools/src/check-handbook.mjs` | **pass** (58×2, 9 generated) | `DEV-WIN-GNU-01` (Node) | worktree, staged | first run reported HB006 for untracked new sources → new files `git add`ed (explicit paths), then OK |
| 2026-09-03 | `node tools/src/generate-handbook.mjs --check` | **pass** (18 pages byte-identical) | `DEV-WIN-GNU-01` (Node) | worktree, staged | — |
| 2026-09-03 | `pnpm run check:consistency` | **pass** | `DEV-WIN-GNU-01` (Node) | worktree, staged | trimmed `lease/personal/P13-T02/hosted-dsh-attempt` row accepted; Personal plan/Gates/leases verified |
| 2026-09-03 | `pnpm -r build` / `pnpm -r test` | **pass** / **pass** | `DEV-WIN-GNU-01` (Node) | worktree, staged | all packages `Done`; adapter suite includes the 12 hosted-attempt-child tests |
| 2026-09-03 | `git diff --check` (worktree + staged) | **pass** | `DEV-WIN-GNU-01` | worktree, staged | no whitespace errors |
| 2026-09-03 | Checkpoint commit + push; Draft PR [#310](https://github.com/agentkernel/cognitive-os/pull/310) opened | recorded | GitHub | `6e5caae4eba476589e97f1b6be5f9d55fe06d268` | network recovered; pre-commit docs-sync gate OK after regenerating fingerprints post-`cargo fmt` |
| 2026-09-03 | Required CI run [33671450531](https://github.com/agentkernel/cognitive-os/actions/runs/33671450531): `verify (ubuntu-latest)` + `verify (windows-latest)` | **fail** (1 test) | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | `6e5caae4` | Rust workspace **built** on both platforms; runtime broker 6/7 pass, `p13_t02_child_direct_provider_is_refused_and_recorded` failed because the test's inline `node -e` argv carried `api.deepseek.com` / `Bearer ` literals and `validate_launch_plan` refused the spawn (`SecretMaterial{argv}`) — the product fail-closed check worked; test defect, fixed in `d2612bb5` (child assembles the strings at runtime) |
| 2026-09-03 | `cargo test -p cognitive-store --test p13_t02_hosted_dsh_attempt --locked` | **pass** 9/9 | `DEV-LINUX-NATIVE-01` `~/cognitiveos-personal-worktrees/p13-t02-6e5caae4` | `6e5caae4eba476589e97f1b6be5f9d55fe06d268` | artifact facts health/update/rollback; unhealthy refuses spawn; persist-before-dispatch → daemon terminal; process death ≠ completion (+ CHECK constraints reject `success`/`completion_claimed=1`/`verification_status='passed'`/DELETE); unknown output → `unknown`; frames never write authority (Employee/Project/stage-test/acceptance unchanged; append-only trigger); crash-shaped rows → `unknown-outcome`; context bounds + secret shape; task channel / assistant / unseated / stale revision / Pi refused. Worktree fetched from GitHub after one `curl 28` timeout retry (host-side transient) |
| 2026-09-03 | Test fix commit pushed | recorded | GitHub | `d2612bb590becef3e5ae140c16604794767fdf9c` | test-only; docs-sync gate `skip` |
| 2026-09-03 | Resume after a second aborted worker: `gh pr checks 310` → **no checks** on `d2612bb5` and PR `mergeable: CONFLICTING` (origin/main moved: P13-T12/D01 PR #308 `main@3680b742`, lease closure `main@84188aac`, docs-only, no migration) | recorded | GitHub / `DEV-WIN-GNU-01` | `d2612bb5` | GitHub could not build the PR merge commit, so no `pull_request` CI ran for the test fix; resolved by merging `origin/main` into the task branch (no rebase / no force push) |
| 2026-09-03 | `git merge origin/main` — 4 docs conflicts resolved (PERSONAL-DEVELOPMENT-PLAN status line + Phase 13 / 合计 counts → P13-T02 **and** P13-T12 in-progress: 167/132/2/1/16; PROGRESS snapshot heading, Active task lease row, Layer 1 table 167/132/2/1/16, Layer 1 prose; handbook `architecture-overview` en + zh-CN fingerprints regenerated); `fill-handbook-fingerprints` (2 pages) → `check-handbook` **pass** (58×2) → `generate-handbook --check` **pass** (18 byte-identical) → `check:consistency` **pass** → `git diff --check` **pass** | **pass** | `DEV-WIN-GNU-01` (Node) | merge commit (below) | sibling ledger rows integrated per PARALLEL-LANES rule 8; no sibling migration landed, v36 number kept |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit. Not `P13-T04` CAS/verifier/outputs, not `P13-T05` Routine arming, not `P13-T08` Settings chrome (engine facts only become readable). Not Installed Agent chrome, not native DSH UI in `/ui/`, not Pi as Member engine. Linux Path B / Linux real spawn ≠ Windows sandbox / ACL / supply-chain qualification. Evaluation routing OFF.

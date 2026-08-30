# P11-T07 Hidden hosted DSH engine — running report

- Task: `P11-T07` / slice `P11-T07/D01`
- Change class: `implementation-only` (v31 managed-child identity + management HTTP start/observe; no `core/specs`, no Lane-CTR, no `/ui/` chrome, no Pi Member engine)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P11-T07/dsh`
- Branch: `personal/P11-T07-dsh`
- Worktree: `D:\agent-kernel-wt-P11-T05` (original `d:\agent-kernel` left dirty; A8 protected)
- Claim ceiling: `hypothesis` (A7: local/CI is not Gate/release/Profile; Linux Path B is not Windows hosted qualification)
- Evaluation routing: **OFF** (`PERSONAL-PERF-EVAL-015` closed)

## Unique next action

Required CI on `21c03171b1f5caa7fdcd231abaadaaa988cbe1e6` is **SUCCESS** (run [33317772618](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618)). `DEV-WINDOWS-NATIVE-OPC-01` hosted E2E remains **not-run**. Linux Path B ≠ Windows hosted qualification. Closure records the pass; this checkpoint commit moves HEAD off `21c03171` and the new SHA needs its own required-ci. This turn does not merge [#287](https://github.com/agentkernel/cognitive-os/pull/287). [2026-08-30-personal-p11-t07-dsh-closure.md](2026-08-30-personal-p11-t07-dsh-closure.md).

## Closed predecessor

`P11-T12` **done**: merged PR [#286](https://github.com/agentkernel/cognitive-os/pull/286) at `main@9e9b18b690cfe63aaedc457bf06d0763965a80fd`. Lease `lease/personal/P11-T12/usage` closed into PARALLEL-LANES §3.1. Honest usage is not a T07 rewrite. `P11-T13`/`T14`/`T15` stay unclaimed/parked.

## Identifier

Hidden engine pin: `cognitiveos.personal.hidden-hosted-dsh/0.1`.

Reused, not rebuilt: Path B `POST /provider/v1/dsh/chat/completions`, `agent://personal/dsh` (proxy routing only — never Employee chrome), Employee `runtime_binding_ref` / `bind_runtime`, T04 `observe_attempt_process_exit` (now a real child-exit UPDATE). Artifact digest pins `528c682e061696f5a160f363f236ecbf53cbd006` (same object as `DSH_PACKAGE_REVISION`). Protocol `akp-http-json-sse`.

Not Installed Agent chrome. Not native DSH UI. Not engine store. Pi is not the Member execution engine.

## Failure-first (this slice)

| ID | Test | Surface |
|---|---|---|
| N1 | digest mismatch rejected | store `p11_t07_digest_mismatch_is_rejected`; HTTP mismatch 422 |
| N2 | protocol mismatch rejected | store `p11_t07_protocol_mismatch_is_rejected` |
| N3 | secret never in child env/argv/logs | store `p11_t07_secret_never_enters_child_env_or_argv`; HTTP body has no `sk-` / `api_key` |
| N4 | unknown child output ≠ success | store `p11_t07_unknown_child_output_is_not_success`; HTTP `child_output: success` 422 |
| N5 | process death does not delete Employee / conversation / Memory | store `p11_t07_process_death_does_not_delete_employee_conversation_or_memory` (extends T04-N1) |
| N6 | Pi is not the Member execution engine | store `p11_t07_pi_is_not_the_member_execution_engine`; `bind_runtime("pi:…")` rejected |
| N7 | task channel cannot bind DSH process | store `p11_t07_task_channel_cannot_bind_hosted_dsh`; HTTP `POST /task/project/v1/dsh.hosted.start` 403 |

## Vertical slice

Attempt-runner `start(...)` is a store+HTTP skeleton (not a full stdio broker). Management `POST /management/project/v1/dsh.hosted.start` persists `p11_hosted_dsh_child` and binds Employee `runtime_binding_ref`. `observe-exit` clears pid and marks `exited`. Isolated spawn fail-closes on Windows GNU (`HOSTED_DSH_WIN_GNU_FENCE`). Daemon Provider proxy remains the only secret-bearing path.

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

Units are appended **immediately** after each finishes. `not-run` is never pass.

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-08-30 | T12 lease close + T07 claim | recorded | docs-only | this commit | `lease/personal/P11-T12/usage` closed after PR [#286](https://github.com/agentkernel/cognitive-os/pull/286) merge `9e9b18b6`; `lease/personal/P11-T07/dsh` active |
| 2026-08-30 | D01 store/HTTP implementation + failure-first tests written | recorded | `DEV-WIN-GNU-01` | this commit | v31 child identity; digest/protocol/secret/unknown-output/Pi/task-channel/process-death; GNU fence |
| 2026-08-30 | `cargo test -p cognitive-store --test p11_t07_hosted_dsh` | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01`; route to CI/Linux |
| 2026-08-30 | `cargo test -p kernel-server --bin kernel-server -- hosted_dsh_start` | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01`; route to CI/Linux |
| 2026-08-30 | `cargo build` / Clippy | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01` |
| 2026-08-30 | Windows hosted sandbox / supply-chain E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` | this commit | Card allows `not-run` until the host route is qualified. Linux Path B ≠ Windows hosted qualification |
| 2026-08-30 | Isolated spawn on `DEV-WIN-GNU-01` | fail-closed (recorded fence) | `DEV-WIN-GNU-01` | this commit | `HostedDshPlane::isolated_spawn_is_fenced()`; start rejects before persist |
| 2026-08-30 | dsh web into `/ui/` / Installed Agent chrome / Pi as Member engine | **not-run** / out of scope | n/a | this commit | T13 owns `/ui/` IA; Pi adapter is not the Member engine |
| 2026-08-30 | `cargo fmt --all` | pass | `DEV-WIN-GNU-01` | this commit | formatting only; no link |
| 2026-08-30 | `node tools/src/generate-handbook.mjs` | pass | local Node | this commit | regenerated `http-api` both locales for `dsh.hosted.start` / `dsh.hosted.observe-exit` |
| 2026-08-30 | `node tools/src/fill-handbook-fingerprints.mjs` | pass | local Node | this commit | store-migrations + daemon-http + mapped fingerprint-only pages |
| 2026-08-30 | `node tools/src/check-consistency.mjs` | pass | local Node | this commit | Personal plan/leases OK including `P11-T07/D01` |
| 2026-08-30 | `node tools/src/check-handbook.mjs` | pass | local Node | this commit | 58×2 locales; coverage/fingerprint OK |
| 2026-08-30 | `node tools/src/generate-handbook.mjs --check` | pass | local Node | this commit | 18 pages byte-identical |
| 2026-08-30 | `cargo test -p cognitive-store --test p11_t07_hosted_dsh` | **pass** 8/8 | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t07-21c03171` | `21c03171b1f5caa7fdcd231abaadaaa988cbe1e6` | Digest/protocol/secret/unknown-output/Pi/task-channel/process-death/GNU-fence. Not B01. Linux Path B ≠ Windows hosted qualification |
| 2026-08-30 | `cargo test -p kernel-server --bin kernel-server -- hosted_dsh_start_persists_binding_and_task_channel_is_forbidden` | **pass** | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t07-21c03171` | `21c03171b1f5caa7fdcd231abaadaaa988cbe1e6` | Management start persists binding; task channel 403; observe-exit HTTP; unknown `child_output: success` 422 |
| 2026-08-30 | Windows hosted sandbox / supply-chain E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` | `21c03171b1f5caa7fdcd231abaadaaa988cbe1e6` | Card allows `not-run` until the host route is qualified. Do not claim Windows sandbox pass |
| 2026-08-30 | `cargo` build/test/clippy | **not-run** | `DEV-WIN-GNU-01` | `21c03171b1f5caa7fdcd231abaadaaa988cbe1e6` | `RUST-LINK-DEV-WIN-GNU-01` |
| 2026-08-30 | B01 guest | **not-run** | `B01-Desktop-Linux-002` | `21c03171b1f5caa7fdcd231abaadaaa988cbe1e6` | Evaluation routing OFF; not a B01 campaign |
| 2026-08-30 | `verify (ubuntu-latest)` | **pass** | `CI-UBUNTU-01` | `21c03171b1f5caa7fdcd231abaadaaa988cbe1e6` | [job 99274255908](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618/job/99274255908) run [33317772618](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618). Build/Test Rust, Clippy `-D warnings`, rustfmt, handbook, consistency **SUCCESS** |
| 2026-08-30 | `verify (windows-latest)` | **pass** | `CI-WINDOWS-MSVC-01` | `21c03171b1f5caa7fdcd231abaadaaa988cbe1e6` | [job 99274255912](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618/job/99274255912) run [33317772618](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618) |
| 2026-08-30 | `required-ci` | **pass** | required-ci | `21c03171b1f5caa7fdcd231abaadaaa988cbe1e6` | [job 99275908890](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618/job/99275908890) run [33317772618](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618). All-green: required-ci + ubuntu + windows |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit. Not T13 `/ui/` IA. Not embedding dsh web into `/ui/`. Not Pi as Member execution engine. Not a full stdio broker. Not T08 Routine, T14/T15. Evaluation routing OFF.

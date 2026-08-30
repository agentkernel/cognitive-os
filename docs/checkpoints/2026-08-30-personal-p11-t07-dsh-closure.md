# P11-T07 Hidden hosted DSH engine closure

- Task: `P11-T07` / slice `P11-T07/D01` (full Phase 11 T07 acceptance)
- Change class: `implementation-only` (v31 managed-child identity + management HTTP start/observe; no `core/specs`, no Lane-CTR, no `/ui/` chrome, no Pi Member engine)
- Branch: `personal/P11-T07-dsh`
- D01 implementation / product HEAD recorded for required-ci: `21c03171b1f5caa7fdcd231abaadaaa988cbe1e6`
- Pull request: [#287](https://github.com/agentkernel/cognitive-os/pull/287) **Draft** (parent flips ready/merge; this checkpoint does not)
- Lease: `lease/personal/P11-T07/dsh` (stays active until parent merge/lease close)
- Required CI on `21c03171`: **SUCCESS** — [required-ci](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618/job/99275908890), [verify (ubuntu-latest)](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618/job/99274255908), [verify (windows-latest)](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618/job/99274255912) on run [33317772618](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618). Incremental log: [report](2026-08-30-personal-p11-t07-dsh-report.md)
- Claim ceiling: `hypothesis`
- Evaluation routing: **OFF**

## Acceptance mapping

D01 covers full Phase 11 T07 close gate including workspace `required-ci` **SUCCESS** on `21c03171`. Windows hosted sandbox/supply-chain E2E and `DEV-WIN-GNU-01` cargo remain honest **not-run**. Linux store 8/8 and named hosted-start HTTP at `21c03171` are **pass**. Linux Path B is not Windows hosted qualification.

| Acceptance item | Evidence |
|---|---|
| Hidden engine pin; not Installed Agent chrome; not native DSH UI | `cognitiveos.personal.hidden-hosted-dsh/0.1`. HTTP `installed_agent: false`. No `/ui/` embed |
| Exact artifact digest pin | `528c682e061696f5a160f363f236ecbf53cbd006` (= `DSH_PACKAGE_REVISION`). Store `p11_t07_digest_mismatch_is_rejected` **pass**; HTTP mismatch **422** |
| Isolated child identity + GNU spawn fail-closed | v31 `p11_hosted_dsh_child`; `runtime_binding_ref` = `hosted-dsh:<digest>:<child_id>`. `HOSTED_DSH_WIN_GNU_FENCE` on `DEV-WIN-GNU-01` |
| Attempt-runner `start` skeleton + daemon Provider proxy | Management `POST /management/project/v1/dsh.hosted.start`. Reuses Path B `POST /provider/v1/dsh/chat/completions`. Not a full stdio broker |
| Protocol pin `akp-http-json-sse` | store `p11_t07_protocol_mismatch_is_rejected` **pass** |
| Secret never in child env/argv | store `p11_t07_secret_never_enters_child_env_or_argv` **pass**; HTTP body has no `sk-` / `api_key` |
| Unknown child output ≠ success | store `p11_t07_unknown_child_output_is_not_success` **pass**; HTTP `child_output: success` **422** |
| Process death does not delete Employee / conversation / Memory | store `p11_t07_process_death_does_not_delete_employee_conversation_or_memory` **pass**. `observe_attempt_process_exit` + `dsh.hosted.observe-exit` |
| Pi is not the Member execution engine | store `p11_t07_pi_is_not_the_member_execution_engine` **pass**; `bind_runtime("pi:…")` rejected |
| Task channel cannot bind DSH process | store `p11_t07_task_channel_cannot_bind_hosted_dsh` **pass**; HTTP `POST /task/project/v1/dsh.hosted.start` **403** |
| Native MCP / base tool / HMR / home patch refused | `reject_native_harness_escape` on start argv/env |
| Linux store T07 focused negatives | **pass** 8/8 at `21c03171` (`DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t07-21c03171`) |
| Linux HTTP start persists binding + task channel forbidden | **pass** 1/1 at `21c03171` (`DEV-LINUX-NATIVE-01`) |
| Windows hosted sandbox / supply-chain E2E | **not-run** (`DEV-WINDOWS-NATIVE-OPC-01`; card allows until qualified). Not claimed as pass |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |
| Workspace `required-ci` on `21c03171` | **SUCCESS** [required-ci](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618/job/99275908890) ([ubuntu](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618/job/99274255908), [windows](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618/job/99274255912)) |

## Validation

| Unit | Result | Env | Revision |
|---|---|---|---|
| store digest/protocol/secret/unknown-output/Pi/task-channel/process-death/GNU-fence | **pass** 8/8 | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t07-21c03171` | `21c03171b1f5caa7fdcd231abaadaaa988cbe1e6` |
| kernel-server `hosted_dsh_start_persists_binding_and_task_channel_is_forbidden` | **pass** 1/1 | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t07-21c03171` | `21c03171b1f5caa7fdcd231abaadaaa988cbe1e6` |
| `check-consistency` / handbook / generate `--check` | **pass** | `DEV-WIN-GNU-01` | D01 commit `21c03171` |
| Windows hosted sandbox / supply-chain E2E | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` unqualified | `21c03171` |
| Rust link on `DEV-WIN-GNU-01` | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | `21c03171` |
| B01 guest | **not-run** | evaluation routing OFF | `21c03171` |
| workspace `required-ci` on product HEAD `21c03171` | **SUCCESS** | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | `21c03171b1f5caa7fdcd231abaadaaa988cbe1e6` |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit, or live `/ui/` IA (A7: local/CI is hypothesis only). Not T13 `/ui/` IA. Not embedding dsh web into `/ui/`. Not Pi as Member execution engine. Not a full stdio broker. Not Windows sandbox install/isolate/rollback of a live DSH package (host E2E **not-run**). Linux Path B is not Windows hosted qualification. Not T02/T08/T14/T15. This checkpoint records workspace `required-ci` **SUCCESS** on `21c03171` and does not ready/merge #287. Live `/ui/` remains Linux 1.0 six-family.

## Remaining parent closure

D01 acceptance mapping for `P11-T07` is recorded at product HEAD `21c03171`, with Linux store 8/8 + HTTP **pass** and workspace `required-ci` **SUCCESS** at that SHA (run [33317772618](https://github.com/agentkernel/cognitive-os/actions/runs/33317772618)). This file does **not** flip PR [#287](https://github.com/agentkernel/cognitive-os/pull/287), merge, or close the lease. Committing this checkpoint moves HEAD off `21c03171`; that new SHA needs its own required-ci before merge.

After the parent confirms required-ci on this checkpoint HEAD, marks #287 ready, and merges:

1. close `lease/personal/P11-T07/dsh`;
2. delete the task branch when safe;
3. do not auto-claim `P11-T02`/`T08`/`T10`/`T11`/`T13`; do not unpark `P11-T14`/`T15`. Do not treat this file as that claim.

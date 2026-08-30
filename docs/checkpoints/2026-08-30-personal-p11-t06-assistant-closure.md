# P11-T06 Hidden Pi Personal Assistant closure

- Task: `P11-T06` / slice `P11-T06/D01` (full Phase 11 T06 acceptance)
- Change class: `implementation-only` (daemon/store/HTTP candidate path; no `core/specs`, no Lane-CTR, no `/ui/` chrome)
- Branch: `personal/P11-T06-assistant`
- D01 implementation revision: `38009eae8dad8a09f7c400f399b9640dd4aebd32`
- rustc E0631 type-coercion revision: `3ff1d615eb1b0c85b0b49113359b5b58c47fa0c2`
- Linux clippy/tests / product HEAD: `845442eac1b8499dec60363f02ffc8d5bae79a85`
- Pull request: [#284](https://github.com/agentkernel/cognitive-os/pull/284) **Draft** (parent flips ready/merge; this checkpoint does not)
- Lease: `lease/personal/P11-T06/assistant` (stays active until parent merge/lease close)
- Required CI on `845442ea`: **SUCCESS** — [required-ci](https://github.com/agentkernel/cognitive-os/actions/runs/33307554027/job/99248695647), [verify (ubuntu-latest)](https://github.com/agentkernel/cognitive-os/actions/runs/33307554027/job/99246714983), [verify (windows-latest)](https://github.com/agentkernel/cognitive-os/actions/runs/33307554027/job/99246715047) on run [33307554027](https://github.com/agentkernel/cognitive-os/actions/runs/33307554027). Incremental log: [report](2026-08-30-personal-p11-t06-assistant-report.md)
- Claim ceiling: `hypothesis`
- Evaluation routing: **OFF**

## Acceptance mapping

D01 covers full Phase 11 T06 acceptance. Host Pi E2E and `DEV-WIN-GNU-01` cargo remain honest **not-run**. Linux crate-scoped clippy/tests at `845442ea` are **pass**; workspace `required-ci` on that SHA is **SUCCESS**.

| Acceptance item | Evidence |
|---|---|
| Hidden engine pin; Pi is not an Installed Agent | `cognitiveos.personal.hidden-pi-assistant/0.1` + exact Pi `0.81.1` + `cognitiveos.private-candidate/1`. HTTP `installed_agent: false` **pass** at `3ff1d615` |
| Candidate-only; change goes through daemon preview | `vertical_turns_register_candidate_and_preview_handoff` **pass**. HTTP `assistant.turn` propose returns `candidate_digest` + `preview_id`, omits Approve / `preview_digest` **pass** at `3ff1d615` |
| Typed provenance; unlabeled / forged source refused | `unlabeled_assistant_candidate_register_is_rejected` **pass**. HTTP unlabeled provenance **422** at `3ff1d615` |
| No authority / SecretStore / archive / Memory write | `assistant_cannot_write_archive_secret_or_authority` **pass**. `draft_apply_targeting_authority_objects_is_rejected` **pass** |
| Closed schema: grant / secret / trigger-arm refused | `closed_schema_rejects_grant_secret_and_trigger_arm` **pass**. HTTP grant field **422** at `3ff1d615` |
| Default-deny tools; ambient shell refused | `default_deny_tools_and_ambient_shell_are_rejected` **pass**. HTTP `tools: ["bash"]` **403** at `3ff1d615` |
| Vertical explain/navigate/research/propose → digest + preview | store 6/6 **pass** at `845442ea`. Research reuses `HttpFetchReadOnly` only |
| Chat has no Approve; task-channel turn refused | store confirm-preview **Forbidden**. HTTP task-channel `assistant.turn` **403** at `3ff1d615` |
| Linux store T06 negatives + vertical path | **pass** 6/6 at `845442ea` |
| Linux crate-scoped Clippy `-D warnings` (store + kernel-server) | **pass** at `845442ea` |
| Host Pi routing / live Pi 0.81.1 E2E | **not-run** (card allows until host Pi routing is qualified; identity pin only) |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |
| Workspace `required-ci` on `845442ea` | **SUCCESS** [required-ci](https://github.com/agentkernel/cognitive-os/actions/runs/33307554027/job/99248695647) ([ubuntu](https://github.com/agentkernel/cognitive-os/actions/runs/33307554027/job/99246714983), [windows](https://github.com/agentkernel/cognitive-os/actions/runs/33307554027/job/99246715047)) |

## Validation

| Unit | Result | Env | Revision |
|---|---|---|---|
| store unlabeled provenance + authority-target apply + no archive/Secret/Memory/authority write + closed schema + default-deny + vertical turns | **pass** 6/6 | `DEV-LINUX-NATIVE-01` | `845442eac1b8499dec60363f02ffc8d5bae79a85` |
| kernel-server `assistant_turn_registers_candidate_and_omits_approve` (422/403 + omit Approve) | **pass** 1/1 | `DEV-LINUX-NATIVE-01` | `3ff1d615eb1b0c85b0b49113359b5b58c47fa0c2` (clippy-only delta to `845442ea`) |
| `cargo clippy -p cognitive-store --all-targets -- -D warnings` | **pass** | `DEV-LINUX-NATIVE-01` | `845442eac1b8499dec60363f02ffc8d5bae79a85` |
| `cargo clippy -p kernel-server --all-targets -- -D warnings` | **pass** | `DEV-LINUX-NATIVE-01` | `845442eac1b8499dec60363f02ffc8d5bae79a85` |
| `check-consistency` / handbook / generate `--check` | **pass** | `DEV-WIN-GNU-01` | D01 commits through `845442ea` |
| Host Pi E2E / live Pi 0.81.1 spawn | **not-run** | host Pi routing unqualified | `845442ea` |
| Rust link on `DEV-WIN-GNU-01` | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | `845442ea` |
| `required-ci` on product HEAD `845442ea` | **SUCCESS** [job 99248695647](https://github.com/agentkernel/cognitive-os/actions/runs/33307554027/job/99248695647) | GitHub Actions | `845442eac1b8499dec60363f02ffc8d5bae79a85` |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit, or live `/ui/` IA (A7: local/CI is hypothesis only). Not T02 (Windows host). Not T05 redo. Not T07 hosted DSH. Not T08 Routine/Trigger. Not T13 right-rail chrome. Pi is not an Installed Agent. Linux Pi pin is not a Windows OPC qualification. Chat has no Approve; draft-apply is not authority-approve. Live `/ui/` remains Linux 1.0 six-family.

## Remaining parent closure

D01 acceptance mapping for `P11-T06` is recorded at product HEAD `845442ea`, with Linux clippy/tests **pass** and workspace `required-ci` **SUCCESS** at that SHA (run [33307554027](https://github.com/agentkernel/cognitive-os/actions/runs/33307554027)). This checkpoint does **not** flip PR [#284](https://github.com/agentkernel/cognitive-os/pull/284), merge, close the lease, or claim `P11-T09`.

After the parent confirms required-ci on `845442ea`, marks #284 ready, and merges:

1. close `lease/personal/P11-T06/assistant`;
2. delete the task branch when safe;
3. **then** claim `P11-T09` (knife `T06→T09`: T06 candidate/preview handoff is the caller for HITL canvas). `P11-T07` and `P11-T02` do not block. `P11-T09` does not depend on `P11-T08` (`implementation_requires`: `P11-T03` + existing preview/Effect/alert/recovery). Do not treat this file as that claim.

# P11-T09 HITL canvas (not Inbox) closure

- Task: `P11-T09` / slices `P11-T09/D01` + `P11-T09/D02` (full Phase 11 T09 acceptance)
- Change class: `implementation-only` (daemon durable ApprovalPreview + StandingApprovalPolicy + grant-expansion; no `core/specs`, no Lane-CTR, no `/ui/` chrome, no chat Approve)
- Branch: `personal/P11-T09-hitl-canvas`
- D01 implementation revision: `8ea2a25b4db064cda8e4effa8a2b2850487b3dee`
- D02 / product HEAD: `381e14c8161ebb65dac3e44785af6bd1213a255e`
- Pull request: [#285](https://github.com/agentkernel/cognitive-os/pull/285) **Draft** (parent flips ready/merge; this checkpoint does not)
- Lease: `lease/personal/P11-T09/hitl-canvas` (stays active until parent merge/lease close)
- Required CI on `381e14c8`: **SUCCESS** — run [33311296122](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122): `resolve validation route` **SUCCESS** [99256719910](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122/job/99256719910), `verify (ubuntu-latest)` **SUCCESS** [99256728125](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122/job/99256728125), `verify (windows-latest)` **SUCCESS** [99256728174](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122/job/99256728174), `required-ci` **SUCCESS** [99258078729](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122/job/99258078729). Incremental log: [report](2026-08-30-personal-p11-t09-hitl-canvas-report.md)
- Claim ceiling: `hypothesis`
- Evaluation routing: **OFF**

## Acceptance mapping

D01+D02 cover full Phase 11 T09 acceptance. Host UI E2E, Settings chrome, and `DEV-WIN-GNU-01` cargo remain honest **not-run**. Linux store 11/11 and named canvas-path HTTP at `381e14c8` are **pass**. Workspace `required-ci` on that SHA is **SUCCESS**.

| Acceptance item | Evidence |
|---|---|
| HITL only on project-center canvas + Today deep-link; not a first-level Inbox queue | Reused T05 announce + deep-link. Management HTTP is the canvas caller (`preview.request` returns `preview_digest`). No independent `#/hitl` route. Host UI E2E **not-run** (T13 chrome) |
| Conversation announces only; chat has no Approve | Store `chat_and_task_channel_cannot_complete_approval` **pass** (in 11/11 at `381e14c8`). HTTP `task_channel_confirm_is_forbidden` **pass** at `8ea2a25b`. Grant-expansion task-channel confirm **403** inside `http_grant_expansion_confirm_returns_digest_on_canvas_path` **pass** at `381e14c8` |
| Durable reject / narrow / confirm; reject leaves receipt | Store `reject_leaves_receipt_and_rejected_digest_is_not_confirmable`, `narrow_mints_new_preview_and_freezes_old_digest`, `confirm_still_works_for_a_fresh_pending_after_reject` **pass** in 11/11. HTTP reject/narrow/confirm **pass** at `8ea2a25b` |
| Stale is mechanical digest mismatch; wrong digest fail-closed | Store `stale_is_mechanical_base_digest_mismatch_not_time`, `wrong_digest_fail_closed_on_confirm_reject_narrow` **pass** in 11/11. HTTP `http_wrong_digest_fail_closed` **pass** at `8ea2a25b` |
| Canvas path can confirm without a chat Approve control | `http_grant_expansion_confirm_returns_digest_on_canvas_path` **pass** at `381e14c8` (`preview_digest` on `preview.request`) |
| StandingApprovalPolicy time-box: `expires_at` required, ≤7d | Store `standing_policy_missing_expires_at_is_rejected`, `standing_policy_over_seven_days_is_rejected` **pass** in 11/11 at `381e14c8` |
| Settings list/revoke on owner management HTTP (no T13 chrome) | Store `standing_policy_lists_and_revokes_on_owner_path` **pass** in 11/11. Settings chrome **not-run** |
| `grant-expansion` subject_kind; reuse request_preview/confirm | Store `grant_expansion_preview_confirmable_on_owner_not_chat`, `unsupported_subject_kind_is_rejected` **pass** in 11/11. HTTP canvas-path confirm **pass** at `381e14c8` |
| Linux store T09 D01+D02 | **pass** 11/11 at `381e14c8` (`DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t09-381e14c8`) |
| Host UI E2E | **not-run** (card allows until qualified; chrome is T13) |
| Settings chrome | **not-run** (T13; HTTP list/revoke only) |
| `DEV-WIN-GNU-01` cargo test / Clippy / link | **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |
| Workspace `required-ci` on `381e14c8` | **SUCCESS** — run [33311296122](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122): ubuntu [99256728125](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122/job/99256728125); windows [99256728174](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122/job/99256728174); required-ci [99258078729](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122/job/99258078729). D01 CI [33310305511](https://github.com/agentkernel/cognitive-os/actions/runs/33310305511) **SUCCESS** at `8ea2a25b` is not merge HEAD |

## Validation

| Unit | Result | Env | Revision |
|---|---|---|---|
| store reject/narrow/stale/wrong-digest/chat-403 + standing policy + grant-expansion | **pass** 11/11 | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t09-381e14c8` | `381e14c8161ebb65dac3e44785af6bd1213a255e` |
| `http_grant_expansion_confirm_returns_digest_on_canvas_path` | **pass** | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t09-381e14c8` | `381e14c8161ebb65dac3e44785af6bd1213a255e` |
| D01 HTTP reject/narrow/wrong-digest/pending-omit/task-403 | **pass** | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t09-8ea2a25b` | `8ea2a25b4db064cda8e4effa8a2b2850487b3dee` (D01; not merge HEAD) |
| `check-consistency` / handbook / generate `--check` / docs-sync-gate | **pass** | `DEV-WIN-GNU-01` | D02 commit `381e14c8` |
| Host UI E2E / Settings chrome | **not-run** | T13 chrome unqualified | `381e14c8` |
| Rust link on `DEV-WIN-GNU-01` | **not-run** | `RUST-LINK-DEV-WIN-GNU-01` | `381e14c8` |
| D01 `required-ci` at `8ea2a25b` | **SUCCESS** (not merge HEAD) | required CI [33310305511](https://github.com/agentkernel/cognitive-os/actions/runs/33310305511) | `8ea2a25b4db064cda8e4effa8a2b2850487b3dee` |
| `verify (ubuntu-latest)` on `381e14c8` | **SUCCESS** [99256728125](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122/job/99256728125) | `CI-UBUNTU-01` | `381e14c8161ebb65dac3e44785af6bd1213a255e` |
| `verify (windows-latest)` on `381e14c8` | **SUCCESS** [99256728174](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122/job/99256728174) | `CI-WINDOWS-MSVC-01` | `381e14c8161ebb65dac3e44785af6bd1213a255e` |
| `required-ci` on product HEAD `381e14c8` | **SUCCESS** [99258078729](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122/job/99258078729) | GitHub Actions run [33311296122](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122) | `381e14c8161ebb65dac3e44785af6bd1213a255e` |

## Explicit non-claims

Not Gate, release, Profile, B01, Windows OPC qualification, Agent-benefit, or live `/ui/` IA (A7: local/CI is hypothesis only). Not T02 (Windows host). Not T05/T06 redo. Not T07 hosted DSH. Not T08 Routine/Trigger. Not T13 right-rail / Settings chrome. Not Inbox L1. Chat has no Approve. Standing list/revoke is HTTP only. `required-ci` green on `381e14c8` is hypothesis only (A7), not Gate. D01 CI green at `8ea2a25b` is not T09 merge HEAD. This checkpoint does not ready/merge #285. Live `/ui/` remains Linux 1.0 six-family.

## Remaining parent closure

D01+D02 acceptance mapping for `P11-T09` is recorded at product HEAD `381e14c8`, with Linux store 11/11, canvas-path HTTP **pass**, and workspace `required-ci` **SUCCESS** on run [33311296122](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122) (ubuntu [99256728125](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122/job/99256728125); windows [99256728174](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122/job/99256728174); required-ci [99258078729](https://github.com/agentkernel/cognitive-os/actions/runs/33311296122/job/99258078729)). This checkpoint does **not** flip PR [#285](https://github.com/agentkernel/cognitive-os/pull/285), merge, close the lease, or claim the next P11 task.

After the parent marks #285 ready and merges:

1. close `lease/personal/P11-T09/hitl-canvas`;
2. delete the task branch when safe;
3. **then** claim the next ready non-overlapping P11 task. Owner knife after T09 is not specified beyond T09 (knife was `T06→T09`). `P11-T04` is **done**, so `P11-T12` deps (`P11-T03` + `P11-T04`) are met; `P11-T02` is also ready and does not block T03. Remaining Phase 11 after T09: typically T12 if ready, else T02 / T08 / T10 (T05 done); T07 waits on T12; T11 waits on T10; T13 is `/ui/` IA. `P11-T14`/`P11-T15` stay **parked** — do not unpark. Do not treat this file as that claim.

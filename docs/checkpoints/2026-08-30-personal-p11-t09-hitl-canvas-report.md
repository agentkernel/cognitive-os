# P11-T09 HITL canvas (not Inbox) — running report

- Task: `P11-T09` / slices `P11-T09/D01` + `P11-T09/D02`
- Change class: `implementation-only` (daemon durable ApprovalPreview + StandingApprovalPolicy + grant-expansion; no `core/specs`, no Lane-CTR, no `/ui/` chrome, no chat Approve)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P11-T09/hitl-canvas`
- Branch: `personal/P11-T09-hitl-canvas`
- Worktree: `D:\agent-kernel-wt-P11-T05` (original `d:\agent-kernel` left dirty on `personal/P11-T04-employee`; A8 protected)
- PR: Draft (this delivery)
- Claim ceiling: `hypothesis` (A7: local/CI is not Gate/release/Profile)
- Evaluation routing: **OFF** (`PERSONAL-PERF-EVAL-015` closed)

## Identifier / reuse

Reused, not rebuilt: v26 `request_preview` / `confirm_preview` / `p11_approval_preview`;
T03 management `preview.request` / `confirm` / `pending-previews` / `preview-detail`;
T05 announce + deep-link only; T06 `draft.apply` is not authority-approve.

D01 adds `reject_preview`, `narrow_preview` (mint new, freeze old as `superseded`
with `superseded_by` via authority migration v29), and management HTTP
`preview.reject` / `preview.narrow`. Chat/task aliases remain 403.

D02 adds authority migration v30: `grant-expansion` subject_kind (preview
CHECK rebuild) plus `p11_standing_approval_policy`. `expires_at` is required
and must be in the future and ≤7 days (`STANDING_POLICY_MAX_TTL_MS`). Settings
list/revoke are management HTTP only; Control Plane chrome is T13 (`not-run`).
`preview.request` returns `preview_digest` so a later canvas can confirm
without a chat Approve control. Host UI E2E **not-run**. No second scheduler.
Not Inbox L1.

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

Units are appended **immediately** after each finishes. `not-run` is never pass.

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-08-30 | T06 lease close + T09 claim | recorded | docs-only | this commit | `lease/personal/P11-T06/assistant` closed after PR [#284](https://github.com/agentkernel/cognitive-os/pull/284) merge `a66faae9`; `lease/personal/P11-T09/hitl-canvas` active |
| 2026-08-30 | D01 store/HTTP implementation + failure-first tests written | recorded | `DEV-WIN-GNU-01` | this commit | Chat/task cannot complete approval; stale is mechanical `base_state_digest` mismatch only; narrow = new preview; reject leaves receipt; wrong digest fail closed |
| 2026-08-30 | `cargo test -p cognitive-store --test p11_t09_hitl_canvas` | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01`; route to CI/Linux |
| 2026-08-30 | `cargo test -p kernel-server --bin kernel-server -- preview` | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01`; route to CI/Linux |
| 2026-08-30 | `cargo build` / Clippy | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01` |
| 2026-08-30 | Host UI E2E | **not-run** | n/a | this commit | Canvas chrome is T13; card allows `not-run` until qualified |
| 2026-08-30 | `cargo fmt --all` | **pass** | `DEV-WIN-GNU-01` | this commit | formatting only; no link |
| 2026-08-30 | `node tools/src/generate-handbook.mjs --check` | **pass** | `DEV-WIN-GNU-01` | this commit | 18 generated pages byte-identical after http-api regen |
| 2026-08-30 | `node tools/src/check-consistency.mjs` | **pass** | `DEV-WIN-GNU-01` | this commit | 275 requirements; leases/plan counts verified |
| 2026-08-30 | `node tools/src/check-handbook.mjs` | **pass** | `DEV-WIN-GNU-01` | this commit | 58×2 locales after staging `p11_t09_hitl_canvas.rs` |
| 2026-08-30 | `node tools/src/docs-sync-gate.mjs --staged` | **pass** | `DEV-WIN-GNU-01` | this commit | store + handbook-itself + personal-2-opc-rebaseline map |
| 2026-08-30 | `cargo test -p cognitive-store --test p11_t09_hitl_canvas` | **pass** 6/6 | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t09-8ea2a25b` | `8ea2a25b4db064cda8e4effa8a2b2850487b3dee` | D01 store filter; not a D02 rerun |
| 2026-08-30 | HTTP: `http_reject_leaves_receipt_and_blocks_old_digest`, `http_narrow_supersedes_old_and_confirm_works_for_new`, `http_wrong_digest_fail_closed`, `pending_previews_omit_digest`, `task_channel_confirm_is_forbidden` | **pass** | `DEV-LINUX-NATIVE-01` `/home/wuz/cognitiveos-personal-worktrees/p11-t09-8ea2a25b` | `8ea2a25b4db064cda8e4effa8a2b2850487b3dee` | kernel-server focused HTTP; D01 names only |
| 2026-08-30 | `cargo test … -- preview_reject` | **not-run** | `DEV-LINUX-NATIVE-01` | `8ea2a25b4db064cda8e4effa8a2b2850487b3dee` | filter matched 0 tests (wrong name); not a fail |
| 2026-08-30 | Host UI E2E | **not-run** | n/a | `8ea2a25b4db064cda8e4effa8a2b2850487b3dee` | T13 chrome; formal card allows `not-run` |
| 2026-08-30 | `cargo test` / Clippy / build | **not-run** | `DEV-WIN-GNU-01` | `8ea2a25b4db064cda8e4effa8a2b2850487b3dee` | `RUST-LINK-DEV-WIN-GNU-01` |
| 2026-08-30 | required CI run [33310305511](https://github.com/agentkernel/cognitive-os/actions/runs/33310305511) | **pass** | `CI-UBUNTU-01` + `CI-WINDOWS-MSVC-01` + required-ci | `8ea2a25b4db064cda8e4effa8a2b2850487b3dee` | ubuntu [99254049060](https://github.com/agentkernel/cognitive-os/actions/runs/33310305511/job/99254049060); windows [99254049044](https://github.com/agentkernel/cognitive-os/actions/runs/33310305511/job/99254049044); required-ci [99255993278](https://github.com/agentkernel/cognitive-os/actions/runs/33310305511/job/99255993278). D01 HEAD only — **do not merge** as T09 close; D02 supersedes |
| 2026-08-30 | D02 store/HTTP: missing `expires_at` rejected; >7d rejected; list/revoke; grant-expansion owner confirm / chat 403 | recorded | `DEV-WIN-GNU-01` | this commit | failure-first tests written; Settings chrome skipped (`clients/pc/web` out of lease) |
| 2026-08-30 | `cargo test -p cognitive-store --test p11_t09_hitl_canvas` (D02 cases) | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01`; route to CI/Linux on new SHA |
| 2026-08-30 | `cargo test -p kernel-server --bin kernel-server -- standing` / `grant` | **not-run** | `DEV-WIN-GNU-01` | this commit | `RUST-LINK-DEV-WIN-GNU-01`; route to CI/Linux on new SHA |
| 2026-08-30 | Host UI E2E / Settings chrome | **not-run** | n/a | this commit | T13; HTTP digest is returned on `preview.request` for a later canvas |
| 2026-08-30 | `cargo fmt --all` | **pass** | `DEV-WIN-GNU-01` | this commit | formatting only; no link |
| 2026-08-30 | `node tools/src/generate-handbook.mjs --check` | **pass** | `DEV-WIN-GNU-01` | this commit | 18 generated pages byte-identical after standing-policy + preview_digest regen |
| 2026-08-30 | `node tools/src/check-handbook.mjs` | **pass** | `DEV-WIN-GNU-01` | this commit | 58×2 locales |
| 2026-08-30 | `node tools/src/check-consistency.mjs` | **pass** | `DEV-WIN-GNU-01` | this commit | after D02 plan/PROGRESS/lease rows |
| 2026-08-30 | `node tools/src/docs-sync-gate.mjs --staged` | **pass** | `DEV-WIN-GNU-01` | this commit | store + kernel-server + handbook + report |

## Unique next action

Push D02 on Draft PR [#285](https://github.com/agentkernel/cognitive-os/pull/285).
Required CI on the **new** SHA (supersedes `8ea2a25b`). Do **not** merge
`8ea2a25b` as T09 close. `DEV-WIN-GNU-01` cargo test remains **not-run**.

## Non-claims

Not T02/T05/T06 redo/T13 chrome. Not Inbox L1, chat Approve, second scheduler,
Vite-as-product, or Lane-CTR. Settings list/revoke is HTTP only. No
Gate/release/Profile/B01/Agent-benefit claim. `8ea2a25b` CI green is D01-only
and is not T09 merge HEAD once D02 is pushed.

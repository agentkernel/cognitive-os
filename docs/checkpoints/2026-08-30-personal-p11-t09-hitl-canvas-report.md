# P11-T09 HITL canvas (not Inbox) — running report

- Task: `P11-T09` / slice `P11-T09/D01`
- Change class: `implementation-only` (daemon durable ApprovalPreview + management HTTP; no `core/specs`, no Lane-CTR, no `/ui/` chrome, no chat Approve)
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

StandingApprovalPolicy time-box and `grant-expansion` subject_kind are **D02**
(CHECK rebuild is not cheap in D01). Host UI E2E **not-run**. No second scheduler.
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

## Unique next action

Push Draft PR. Required CI / Linux focused store+HTTP. Do not merge until
acceptance. `DEV-WIN-GNU-01` cargo test remains **not-run**.

## Non-claims

Not T02/T05/T06 redo/T13 chrome. Not Inbox L1, chat Approve, second scheduler,
Vite-as-product, Lane-CTR, StandingApprovalPolicy time-box (D02), or
`grant-expansion` subject_kind. No Gate/release/Profile/B01/Agent-benefit claim.

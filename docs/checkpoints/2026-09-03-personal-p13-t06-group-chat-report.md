# P13-T06 Project group chat + manager/member routing — running report

- Task: `P13-T06` / slice `P13-T06/D01`
- Change class: `implementation-only` (store / daemon HTTP / `clients/pc/web` right rail; additive authority migration for the chat turn ledger and two ApprovalPreview subject kinds; no `core/specs`, no Lane-CTR; the P11-T05 `cognitiveos.personal.conversation-archive/0.1` identifier is reused, not reinterpreted)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P13-T06/group-chat`
- Branch: `personal/P13-T06-group-chat` (worktree `D:\agent-kernel-wt-p13-t06`, rebased onto `origin/main@327478d4` after P13-T05 lease close)
- Siblings: `P0-T01/D02` (toolchain, Draft PR #314) runs concurrently on its own lease / worktree; shared registration files are edited additively
- PR: Draft PR [#316](https://github.com/agentkernel/cognitive-os/pull/316) (Draft until every acceptance item is mapped)
- Claim ceiling: `hypothesis` (A7: local / CI / Linux-native evidence is not Gate / release / Profile; Windows-native cells stay `not-run` until `P13-T13`)
- Evaluation routing: **OFF**

## Identifier

Group-chat turns and speech records reuse the P11-T05 archive envelope
`cognitiveos.personal.conversation-archive/0.1`. Owner turns land in the new
`p13_project_chat_turn` ledger (authority migration **v39**; T05 took v38;
owner-authored, mention / routing / candidate digest / preview id / receipt
columns; `approve_attempted` CHECK = 0 so the schema itself cannot record a
chat Approve). Manager and Member speech continue to land through
`ConversationStore::land_speech` → `EmployeeStore::route_speech`
(manager-default; Member whitelist `deliverable` / `handoff` / `blocked` /
`decision-request` or mentioned), so the speech rules are enforced by daemon
record kinds, not by the client.

New ApprovalPreview subject kinds: `plan-revision` (subject_ref = chat turn
id; Confirm applies the candidate stage list as a new PlanRevision through
`apply_plan_revision_locked`) and `task-revision` (subject_ref = chat turn id;
Confirm re-materializes the current plan with only the mentioned Member's
responsible stage objective revised). Both are announced in chat and confirmed
only on the Projects canvas (`confirm` / `preview.reject` / `preview.narrow`).

## Recovery

A stale merge left UU files with no `MERGE_HEAD`. Implementation was backed up
to `%TEMP%\p13-t06-impl-backup`, the worktree was reset to `origin/main@327478d4`
(T05 on tree), and T06 sources were remapped to v39 and wired onto T04/T05
siblings (preview kinds keep `run-acceptance` / `external-send`; chat routes
sit beside `routine.runs` / `today.overview`).

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

Units are appended **immediately** after each finishes. `not-run` is never pass.

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-03 | worktree recover + v39 remap + lease claim | recorded | docs-only | uncommitted | Recovered onto `origin/main@327478d4`. Migration `PROJECT_CHAT_SCHEMA_V39` / `MigrationPlanEntry::new(39, …)`. Lease `lease/personal/P13-T06/group-chat` active. Design: manager / Member replies are daemon-composed (`announce` / receipt); real manager reasoning through a hosted DSH Attempt is **not** claimed. |
| 2026-09-03 | local Windows GNU `cargo test` / `clippy` | not-run | `DEV-WIN-GNU-01` | — | `RUST-LINK-DEV-WIN-GNU-01`; route to `DEV-LINUX-NATIVE-01` / required CI. |
| 2026-09-03 | Dual Track TS (`clients/pc/web`) | pass | `DEV-WIN-GNU-01` | uncommitted | vitest **64/64 files, 483/483 tests** including `projectChat.test.ts`, `projectGroupChat.test.tsx`, `normalize.test.ts`. |
| 2026-09-03 | Checkpoint commit + Draft PR | recorded | `DEV-WIN-GNU-01` | `6ed02688` | Implementation + failure-first store 8 tests / kernel-server 6 tests / Dual Track TS committed; Draft PR [#316](https://github.com/agentkernel/cognitive-os/pull/316). |
| 2026-09-03 | Live request-line matcher pin (`parse_route` trim + exact; `project_chat_routes_match_the_live_request_line_shape`) | recorded | `DEV-WIN-GNU-01` | uncommitted | P13-T05 lesson: live daemon dispatches `"<METHOD> <path> "`; in-process tests used untrailed strings. Matcher now strips query + whitespace and refuses suffixed paths. `cargo fmt --all -- --check` **pass**. Rust cargo test **not-run** here (`RUST-LINK-DEV-WIN-GNU-01`). |
| 2026-09-03 | `DEV-LINUX-NATIVE-01` store / kernel-server / live E2E | not-run | Linux | — | Requires the matcher-pin revision after push. |

## Unique next

1. Push the live-shape pin; required CI on the new HEAD (PR #316).
2. Exact-revision Linux: store `p13_t06_project_chat` + `p1_t01_layout_migrations`, kernel-server `project_chat` (incl. live-shape pin), clippy on touched crates, live daemon negatives (chat Approve, secret, cross-Project, task-channel, trailing-space request line).
3. Map every D01 acceptance item, then ready/merge. Do not claim P13-T07.

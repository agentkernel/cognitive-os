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
| 2026-09-03 | `DEV-LINUX-NATIVE-01` store `p13_t06_project_chat` + `p1_t01_layout_migrations` | **pass** 8/8 + 8/8 | Linux | `dd15dc51` | worktree `~/cognitiveos-personal-worktrees/p13-t06-dd15dc51` (shallow clone, dirty=0); `CARGO_TARGET_DIR` reused from `p13-t05-ecd35ab0/target`. |
| 2026-09-03 | `DEV-LINUX-NATIVE-01` kernel-server `project_chat` | **fail** 6/7 | Linux | `dd15dc51` | `manager_mention_…_canvas_confirm_applies` asserted owner then manager; same `created_at` tie-break was lexicographic `conv-*` < `turn-*`, so the manager announce leapfrogged. Live-shape pin **pass**. |
| 2026-09-03 | Fix: same-timestamp owner-message ranks before speech; store test `p13_t06_same_timestamp_owner_turn_precedes_manager_reply` | recorded | `DEV-WIN-GNU-01` | uncommitted | failure-first from Linux; `cargo fmt --all -- --check` after fmt. |
| 2026-09-03 | Required CI run [33730555974](https://github.com/agentkernel/cognitive-os/actions/runs/33730555974) at `6ed02688` | **fail** (docs) | `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | `6ed02688` | `tools` `check-consistency`: (1) formal-plan 合计 167/136/2/1/12 vs task rows 167/137/2/1/11; (2) T06 lease listed `docs/plan/PARALLEL-LANES.md` as a writable path; (3) Current snapshot Active task lease row did not backtick-reference `lease/personal/P13-T06/group-chat`. Not a Rust/product failure. Matcher pin `dd15dc51` was already pushed; CI rollup on that SHA was empty at check time. |
| 2026-09-03 | Local `node tools/src/check-consistency.mjs` after lease/progress/totals repair | **pass** | `DEV-WIN-GNU-01` (Node) | worktree, uncommitted | Removed ledger self-ownership; Active task lease now references both `lease/personal/DOC-PERSONAL-2.0-OPC-REFRAME/product-prototype-docs` and `lease/personal/P13-T06/group-chat`; 合计 already 167/137/2/1/11. |
| 2026-09-03 | `DEV-LINUX-NATIVE-01` GitHub `git fetch` into `p13-t05-ecd35ab0` (direct + `http://127.0.0.1:7890` / `7899`) | fail (pack) | `DEV-LINUX-NATIVE-01` | — | `fatal: pack has 26 unresolved deltas` / `fetch-pack: invalid index-pack output`. Did not use `/home/wuz/agent-kernel`. Proxy probe 200. Host disk 94% (28G free). |
| 2026-09-03 | Reused existing Linux clone `~/cognitiveos-personal-worktrees/p13-t06-dd15dc51` | recorded | `DEV-LINUX-NATIVE-01` | `dd15dc51` | Independent `.git`, `HEAD=dd15dc51`, dirty=0. `CARGO_TARGET_DIR` → T05 `target/`. This session independently re-ran store tests: **pass** 8/8 + 8/8. |
| 2026-09-03 | Linux retest at `08aa2928` (bundle `HEAD --not dd15dc51` after GitHub pack fetch failed on the T05 object store) | recorded | `DEV-LINUX-NATIVE-01` | `08aa2928` | clone already at the SHA (dirty=0). |
| 2026-09-03 | `cargo test -p cognitive-store --test p13_t06_project_chat --test p1_t01_layout_migrations` | **pass** 9/9 + 8/8 | `DEV-LINUX-NATIVE-01` | `08aa2928` | includes `p13_t06_same_timestamp_owner_turn_precedes_manager_reply`. |
| 2026-09-03 | `cargo test -p kernel-server --bin kernel-server -- --test-threads=1 project_chat` | **pass** 7/7 | `DEV-LINUX-NATIVE-01` | `08aa2928` | previously failing `manager_mention_…_canvas_confirm_applies` green; live-shape pin green. |
| 2026-09-03 | `cargo clippy -p cognitive-store -p kernel-server --all-targets --locked -- -D warnings` | **fail** | `DEV-LINUX-NATIVE-01` | `08aa2928` | `clippy::type_complexity` at `project_chat.rs` `load_candidate_locked` (`Option<(String, Option<String>, Option<String>, Option<String>)>`). |
| 2026-09-03 | merge `origin/main@7fe3b839` (T05 lease-close totals 137/1 on main) | recorded | docs-only | `06f49b04` | Conflict only in `PERSONAL-DEVELOPMENT-PLAN.md`. Kept T06 in-progress + v39; T05 **done**/lease closed; this-branch 合计 167/137/2/1/11 (T06+T12 in-progress). T04 v37 / T05 v38 / T06 v39 unchanged. |
| 2026-09-03 | clippy `ChatCandidateColumns` alias | recorded | `DEV-WIN-GNU-01` | `aecc3fc4` | Named the four-column tuple so `-D warnings` no longer trips `type_complexity`. |
| 2026-09-03 | `DEV-LINUX-NATIVE-01` store + kernel-server + clippy at `aecc3fc4` | **pass** 9/9+8/8, 7/7, clippy 0 | `DEV-LINUX-NATIVE-01` | `aecc3fc4` | clone `p13-t06-dd15dc51` fetched via host `:7890`; dirty=0. |
| 2026-09-03 | Required CI [33733963941](https://github.com/agentkernel/cognitive-os/actions/runs/33733963941) ubuntu at `aecc3fc4` | **fail** | `CI-UBUNTU-01` | `aecc3fc4` | `p11_t10_authority_sqlite_omits_secret_shape_bytes_after_import`: v39 CHECK literals `task-revision` / `member-task-revision` persist `sk-` in `sqlite_master` (Vault `task-contract` precedent). |
| 2026-09-03 | v39 CHECK concatenates `task`/`revision` so sqlite_master omits `sk-` | recorded | `DEV-WIN-GNU-01` | uncommitted | Product kind names unchanged. New store lock `p13_t06_authority_sqlite_omits_sk_substring_after_v39`. |

## Unique next

1. Push the sqlite `sk-` CHECK fix; keep PR #316 Draft.
2. Linux store + `p11_t10_vault` + clippy at the new SHA, then live daemon negatives.
3. Map every D01 acceptance item after required CI is green. Do not claim P13-T07.

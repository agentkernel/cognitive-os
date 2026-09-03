# P13-T06 Project group chat + manager routing — closure

- Task: `P13-T06` **done** / slice `P13-T06/D01` **done** (single Delivery Slice)
- Change class: `implementation-only` (no `core/specs`, no Lane-CTR, additive authority migration **v39** `p13_project_chat_turn` + `p11_approval_preview` rebuild that keeps every v30 / v37 kind and adds `plan-revision` / `task-revision`; T04 remains `ATTEMPT_ARTIFACT_SCHEMA_V37`; T05 remains `ROUTINE_ARMING_SCHEMA_V38`; no contract or negative weakened)
- Lease: `lease/personal/P13-T06/group-chat` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P13-T06-group-chat` (worktree `D:\agent-kernel-wt-p13-t06`) → PR [#316](https://github.com/agentkernel/cognitive-os/pull/316)
- Required CI: [33742029203](https://github.com/agentkernel/cognitive-os/actions/runs/33742029203) **SUCCESS** at fold HEAD `6bc56525` (resolve 2s, ubuntu 4m19s, windows 14m5s, required-ci 3s). Live-validated implementation revision `7ae9db50` required CI [33734948118](https://github.com/agentkernel/cognitive-os/actions/runs/33734948118) **SUCCESS**.
- Running report: [P13-T06 report](2026-09-03-personal-p13-t06-group-chat-report.md)
- Claim ceiling: `hypothesis`. Linux native + ordinary CI evidence closes "the implementation exists" only (Phase 13 hard gate 6). No Gate / release / Profile / B01 / Windows / Agent-benefit / conversation-quality claim. Manager / Member replies are daemon-composed announce / receipt record kinds; hosted DSH manager reasoning is **not** claimed.
- Validated implementation revision: `7ae9db50` (store `p13_t06` 10/10 + `p11_t10_vault` 9/9 + migrations 8/8, kernel-server `project_chat` 7/7, clippy `-D warnings` clean, **live daemon E2E 7/7** on `DEV-LINUX-NATIVE-01`).

## 1. Acceptance mapping (formal plan P13-T06 card + Delivery Slice `P13-T06/D01`)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| Group chat (Owner / manager / Members) layered against the Personal Assistant conversation | web `ProjectGroupChat.tsx` + `AssistantRail.tsx`; management `chat.post` / `chat.thread` vs `assistant.turn`; daemon `/ui/` is the product origin | web `projectGroupChat.test.tsx` (group vs assistant layering; no Approve / Confirm in chat); HTTP task-channel aliases 403 | Dual Track TS; live task-channel 403 `PROJECT_CHAT_CHANNEL_FORBIDDEN` |
| `@manager` → daemon PlanRevision **candidate** → canvas preview; chat never applies | store `post_turn` mints `plan-revision` + digest-bound preview; `confirm_chat_candidate_locked` runs only from canvas Confirm (`apply_plan_revision_locked`) | store `p13_t06_manager_mention_registers_plan_revision_candidate_then_canvas_confirm_applies`; HTTP `manager_mention_…_canvas_confirm_applies` | Linux store + HTTP; live `@manager` preview 200, `chat_approve: false` |
| `@member` routes only that Member's Task (stage-bounded `task-revision`) | store mints `task-revision` only for the mentioned Member's responsible stage | store `p13_t06_member_mention_routes_only_that_members_task`; HTTP `member_mention_routes_only_that_members_task` | Linux store + HTTP |
| Speaking rules are daemon record kinds (manager-default; Member proactive only mentioned / delivering / handoff / blocked / decision-request) | manager / Member speech still lands through P11-T05 `land_speech` / `route_speech`; chat reply kinds are announce / receipt | store `p13_t06_speech_rules_are_record_kinds` | Linux store 10/10 |
| `@` inserts only into the unsent draft | web `insertMention` / `chatDraftReady`; chips edit draft only | web `projectChat.test.ts`, `projectGroupChat.test.tsx` | Dual Track TS |
| Chat has no Approve | schema `approve_attempted` CHECK=0; store `approve_from_chat` refusal; HTTP approve-shaped keys 403 before any write | store `p13_t06_chat_has_no_approve`; HTTP `approve_shaped_chat_bodies_are_refused_before_any_write` | live Approve 403 `CHAT_APPROVE_FORBIDDEN` |
| SecretStore takeover: secret-shaped chat refused, pointed at Settings | store refuses before any row; HTTP 422 + `settings_route: #/settings` | store `p13_t06_secret_in_chat_refused_and_never_persisted`; HTTP `secret_shaped_chat_is_refused_…` | live secret 422 `CHAT_SECRET_SHAPED_REFUSED` |
| Cross-Project read / Member-to-Member authority transfer fail closed | store + HTTP 403; thread requires a bound `limit` | store `p13_t06_cross_project_read_and_route_refused`, `p13_t06_chat_cannot_transfer_authority_between_members`; HTTP `cross_project_thread_read_and_member_route_are_forbidden` | live cross-thread / cross-member 403 `PROJECT_FORBIDDEN`; unbounded thread 422 |
| Live request line (`"<METHOD> <path> "`) matches | `parse_route` trim + exact (T05 trailing-space lesson) | HTTP `project_chat_routes_match_the_live_request_line_shape` | live HTTP is the same dispatcher |
| Conversation is never completion | archive append / speech ≠ Task complete; reads bounded | store `p13_t06_conversation_is_not_completion_and_reads_are_bounded` | Linux store |
| Same-timestamp owner turn precedes manager reply | `read_thread` `thread_tie_rank` | store `p13_t06_same_timestamp_owner_turn_precedes_manager_reply` | Linux store (fixed the 6/7 kernel-server fail at `dd15dc51`) |
| v39 CHECK SQL does not embed `sk-` in `sqlite_master` | `'task' \|\| '-' \|\| 'revision'` (Vault `task-contract` precedent) | store `p13_t06_authority_sqlite_omits_sk_substring_after_v39`; sibling `p11_t10_authority_sqlite_omits_secret_shape_bytes_after_import` | Linux 9/9 vault + 10/10 chat; ubuntu CI fail at `aecc3fc4` recorded and fixed |

关闭门 (plan.md card), sentence by sentence: (1) group chat layered against the assistant conversation — **true** (web + management HTTP); (2) `@manager` → candidate → preview — **true** (store / HTTP / live preview; canvas Confirm is the only PlanRevision writer); (3) `@member` routes only that Member's Task — **true**; (4) member speech rules — **true** (daemon record kinds, not a client filter); (5) `@` only enters the unsent draft — **true** (web); (6) chat has no Approve — **true** (schema CHECK + HTTP 403 + live). Manager / Member replies are daemon-composed announce / receipt; a hosted DSH Attempt that "thinks like the manager" is **not** claimed.

Drift negatives from the card, all refused / never produced: chat Approve (403 / CHECK=0), Member-to-Member authority transfer (403), casual Member speech (record-kind router), `@` writing authority (draft only), Conversation as completion (store lock), cross-Project read (403), secret in chat (422 + Settings).

## 2. Validation summary

| Environment | Result |
|---|---|
| `DEV-WIN-GNU-01` | `cargo fmt --all`, Dual Track TS (`clients/pc/web` 64 files / 483 tests at `6ed02688`), `check:consistency`, handbook check set, docs-sync gate — **pass**; cargo build/test/clippy **not-run** (`RUST-LINK-DEV-WIN-GNU-01`) |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | [33730555974](https://github.com/agentkernel/cognitive-os/actions/runs/33730555974) **fail** at `6ed02688` (consistency / lease path — fixed at `08aa2928`); [33733963941](https://github.com/agentkernel/cognitive-os/actions/runs/33733963941) ubuntu **fail** at `aecc3fc4` (`p11_t10` sqlite `sk-` scan — fixed at `e44f43ee` / `7ae9db50`); [33734948118](https://github.com/agentkernel/cognitive-os/actions/runs/33734948118) **SUCCESS** at `7ae9db50`; fold-head [33742029203](https://github.com/agentkernel/cognitive-os/actions/runs/33742029203) **SUCCESS** at `6bc56525` |
| `DEV-LINUX-NATIVE-01` (exact pushed revisions, clone `~/cognitiveos-personal-worktrees/p13-t06-dd15dc51`) | at `7ae9db50`: store 10/10 + vault 9/9 + migrations 8/8, kernel-server 7/7, clippy clean; **live daemon E2E 7/7** on `127.0.0.1:48786` (scratch seed `~/cos-wt/p13t06-seed` outside the repo; G1 + plan + seated A/B; product HTTP only) |
| `DEV-WINDOWS-NATIVE-OPC-01` | **not-run** (not provisioned); rendered `/ui/` review is `P13-T12/D02` |
| `B01-Desktop-Linux-002` | **not-run** (no guest `/ui/` deploy in this task) |

## 3. Non-claims

Not P13-T04 (CAS / outputs), not P13-T05 (routine arming / runs), not P13-T07 Knowledge/Memory auto-admission (T06 is a **source**, not a mutex), not P13-T11 reflection, not P13-T13 Windows qualification. No chat Approve / Confirm / Publish control. No second conversation projection. No Gate / release / Profile / B01 / Windows / Agent-benefit claim. Product HTTP still has no first PlanRevision-apply path other than canvas Confirm (the G1+plan+roster fixture for live E2E is a scratch crate, same recorded P13-T05 gap).

## 4. Cleanup

Linux: E2E daemon stopped, runtime root `…/p13-t06-7ae9db50-runtime` removed; port 48786 released; other tasks' daemons (48181 / 39245 / 48681) untouched. Local task branch / worktree deleted after merge; `main` fast-forwarded.

## 5. Unique next

Do **not** claim sibling-owned `P13-T07` / `T08` / `T10`. Parent next ready card: **`P13-T09`**. `P13-T11` is also ready (T05/D01 done). `P13-T12/D02` still waits T07/D01 + T08/D02.

# P14-T04 member join Dual Track — running report

- Task / slice: `P14-T04/D02` guest `/ui/` J4 (D01 Dual Track local pass)
- Lease: `lease/personal/P14-T04/member-join`
- Branch: `personal/P14-T04-member-join`
- Draft PR: [#331](https://github.com/agentkernel/cognitive-os/pull/331)
- Implementation revision: `fe498997` (D01 Dual Track). Docs-only report HEAD was `3a39614e`.
- Required CI at `3a39614e`: [33987473832](https://github.com/agentkernel/cognitive-os/actions/runs/33987473832) **SUCCESS** (resolve, ubuntu, windows, required-ci)
- Change class: `implementation-only` (Dual Track process-ring ids become PlanRevision `responsible_slot`; write join seats those slots; no `core/specs`; no numbered migration — v41 remains reserved). Handbook: `dev.store-migrations` + `dev.daemon-http-surface` + regenerated `ref.http-api` (both locales).
- Claim ceiling: `hypothesis`
- Product origin: daemon `/ui/` (`http://127.0.0.1:48681/ui/`) — Vite is not the product source
- Guest: `B01-Desktop-Linux-002` `hal9001@192.168.123.160` runtime `/home/hal9001/p13-main-711a5a7c/`
- Evaluation routing: **OFF**
- Do not claim T05/T06/T07/T08. T05 is already claimed (Draft PR [#332](https://github.com/agentkernel/cognitive-os/pull/332), `d72c2847`). T02/T03/T07/T08 remain **done**.

## Failure-first (D01)

Observed fail on current `main@ed893951` (T03 Dual Track collapsed `rights=owner` into one slot): Dual Track activation minted three `owner` slots, so joining `collect`/`analyze`/`draft` was `roster missing slot coverage`. Fake `manager` join and G1-without-plan already fail-closed.

Fix: `parse_dual_track_stages` mints `responsible_slot = stage_id`. `rights=` stays Owner access in the objective text, not a seating slot. Charter blob now also records `slot=${ring.id}`.

## Local development evidence (MSVC override; not supported CI)

- `rustc -vV` host `x86_64-pc-windows-msvc`; `CARGO_PROFILE_DEV_DEBUG=0`
- `cargo test -p cognitive-store --test p14_t04_member_join --locked -- --test-threads=1` **5/5 pass** (`dual_track_activation_mints_responsible_slots`, `write_join_seats_members_on_plan_revision_slots`, `no_slot_fake_join_is_refused`, `surplus_slot_join_does_not_seat`, `chat_approve_must_not_join`)
- `cargo test -p kernel-server dual_track_http_join --locked -- --test-threads=1` **1/1 pass** (`dual_track_http_join_seats_ring_slots_and_refuses_chat`: axis collect/analyze/draft, fake `manager` 422, task-channel register 403, management register+seat 3 seated)
- `cargo test -p cognitive-store --test p14_t03_write_live_project --locked -- --test-threads=1` **4/4 pass** (T03 regression)
- Dual Track TS `pnpm test` in `clients/pc/web` **73 files / 529 tests** (includes Dual Track ring-slot Write join + `uniqueResponsibleSlots` collect/analyze/draft)
- `cargo fmt --all -- --check` **pass**

## Units

| Unit | Result | Evidence |
|---|---|---|
| D01 Dual Track (store/HTTP/TS) | pass | store **5/5**; kernel-server HTTP **1/1**; T03 regression **4/4**; web **73/529**; local MSVC development evidence only |
| Required CI at `3a39614e` | pass | [33987473832](https://github.com/agentkernel/cognitive-os/actions/runs/33987473832): resolve / ubuntu / windows / required-ci **SUCCESS** |
| Exact-revision Linux build | pass | `wuz@192.168.1.2` worktree `/home/wuz/cognitiveos-personal-worktrees/p14-t04-fe498997`; `kernel-server` ELF 42795608; UI dist `index-CW6on2hU.js` |
| Guest daemon replace on 48681 | pass | PID 2653663 `kernel-server --personal --bind 127.0.0.1:48681`; product binary `fe498997`; `/ui/` GET 200 serves `index-CW6on2hU.js`. dsh web on 3080 after replace. Left `:48181` untouched (PID 166715). |
| J0 gate + unauthenticated fail-closed | pass | Empty Issue → `management HTTP 401; task HTTP 401. Bootstrap discarded.` Gate remained. Same-origin one-shot `/ui/.boot-once` fill (file deleted immediately; secret not in Git/chat/report). Header `principal://local/owner · mgmt+task`. Bare `GET /management/project/v1/list` without bearer stays **401**. Empty JSON `/local/session` **400**. |
| J1 regression wizard ①–⑤ | pass | `#/projects/new` Dual Track title `P14-T04 D02 wizard join live` → ② `确认这一环` ×3 → `确认总目标与项目触发` → `进入 ③` → ③ `创建岗位` + model `draft-bound` + six slots + `确认就位` ×3 → ④ `开始测` / `记录通过` / `通过，下一环` ×2 + `末环通过，进入 ⑤` → ⑤ `开始联调` / `核对通过` / `Request preview` minted `draft-01a0731e-ce8f-7f81-abaf-db8410a63404` / `preview-01a0731e-cf02-74c3-93b7-462e3cc8441f` / **Write Project** navigated to Today. Bundle `index-CW6on2hU.js`. Vite not used. |
| J1 list/detail after Write | pass | List row `project-01a0731e-cfb2-72b2-9025-5e8372f45dc1` **active** title **P14-T04 D02 wizard join live** (not `unknown`). Detail Plan revision `plan-01a0731e-cfb5-7611-bb17-4c55852933d9`. Axis slots **collect, analyze, draft** (not collapsed `owner`). |
| J4 member join | pass | `#/projects/project-01a0731e-cfb2-72b2-9025-5e8372f45dc1/members/new`. Page showed slots `collect, analyze, draft`. Chat “Approve this join” posted conversational `no-current-manager`; roster stayed empty. **Write join** seated three employees (`employee-01a07320-5d58-…`, `employee-01a07320-5d62-…`, `employee-01a07320-5d6d-…`); `@ collect` / `@ analyze` / `@ draft` appeared. Guest corroboration after the walk: `state=active`, `slots=collect,analyze,draft`, `seated=3`. |
| J10 no X/Twitter P0 hero | pass | CDP `twitter=false` on Today, Projects, and Knowledge. Bundle `index-CW6on2hU.js`. |
| J18 identity | pass | `#/session` Session page; principal field `principal://local/owner`; header `principal://local/owner · mgmt+task`; Clear memory session present. Bootstrap copy is not a Provider key. |
| J19 retired routes | pass | `#/inbox`, `#/team`, `#/hitl/prev-1`, `#/home`, `#/work` each `No such route` / “This address does not exist in the Control Plane.” |
| Windows native chrome JOURNEY | not-run | Walk used local Cursor browser against forwarded guest `/ui/`, not Windows-native daemon chrome. |

## Unique next

Ready/merge PR [#331](https://github.com/agentkernel/cognitive-os/pull/331) on the closure HEAD after required CI. T05 is already claimed (PR [#332](https://github.com/agentkernel/cognitive-os/pull/332)). T06 is already claimed (PR [#333](https://github.com/agentkernel/cognitive-os/pull/333)). Unique next: T05/D02 and/or T06/D02 on freed guest `:48681`. Do not claim T07/T08.

# P14-T03 Write Project titled live Project — running report

- Task / slice: `P14-T03/D02` guest `/ui/` (D01 Dual Track local pass)
- Lease: `lease/personal/P14-T03/write-live-project`
- Branch: `personal/P14-T03-write-live-project`
- Draft PR: [#330](https://github.com/agentkernel/cognitive-os/pull/330)
- Fold HEAD: `1d50fcc9` (`origin/main@2d92dd16` P14-T07)
- Validated HEAD before closure: `625e9ccd` required CI [33983778699](https://github.com/agentkernel/cognitive-os/actions/runs/33983778699) **SUCCESS**
- Change class: `implementation-only` (G1 Write Project durable title + PlanRevision axis + leave `creating`; no `core/specs`; no numbered migration — v41 remains reserved)
- Claim ceiling: `hypothesis`
- Product origin: daemon `/ui/` (`http://127.0.0.1:48681/ui/`) — Vite is not the product source
- Guest: `B01-Desktop-Linux-002` `hal9001@192.168.123.160` runtime `/home/hal9001/p13-main-711a5a7c/`
- Evaluation routing: **OFF**
- Do not claim T05/T06/T07/T08. T02/T07/T08 remain **done**. After T03 close, claim T04.

## Units

| Unit | Result | Evidence |
|---|---|---|
| Claim + T02 lease close | pass | this branch from `origin/main@c9bb291d`; T02 row → PARALLEL-LANES §3.1 |
| Failure-first Dual Track (empty title / `unknown` / leave `creating` + axis / no-axis still live) | fail then pass | observed 4/4 fail on current `creating` G1; after `activate_locked` Dual Track path: `cargo test -p cognitive-store --test p14_t03_write_live_project --locked -- --test-threads=1` **4/4 pass** (local MSVC host `x86_64-pc-windows-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`; development evidence only) |
| Behavior change | pass | Dual Track titled+process → `active` + `accepted_at` + PlanRevision + Owner `title_summary`; empty/`unknown`/empty `process:` refuse with no row; G1 without `process:` stays `creating`. HTTP list/detail use daemon title. Local: P11-T03 store 19/19; kernel-server `write_project_http_*` 2/2 + `g1_confirm_mints_creating_project` pass; web `projects.test.ts` 5/5 |
| Fold `origin/main@2d92dd16` (P14-T07) | pass | merge `1d50fcc9`; last-merger keeps T02/T07/T08 **done** and T03 **in-progress**. Settings/Knowledge/wizard not reverted. |
| Required CI at fold `1d50fcc9` | fail | run [33982467722](https://github.com/agentkernel/cognitive-os/actions/runs/33982467722): tools `check.test.mjs` — blank line between consecutive active lease table rows left DOC-REFRAME in the fixture after inject. Fix: adjacent active rows. |
| Lease-table CI fix `625e9ccd` | pass | docs-only (PARALLEL-LANES + this report). Product SPA/kernel unchanged vs `1d50fcc9`. |
| Required CI at `625e9ccd` | pass | run [33983778699](https://github.com/agentkernel/cognitive-os/actions/runs/33983778699): resolve **SUCCESS**; ubuntu **SUCCESS**; windows **SUCCESS**; required-ci **SUCCESS**. |
| Exact-revision Linux build | pass | `wuz@192.168.1.2` worktree `/home/wuz/cognitiveos-personal-worktrees/p14-t03-1d50fcc9`; `kernel-server` ELF 42798272; UI dist `index-CvtgBWRI.js` |
| Guest daemon replace on 48681 | pass | PID 2632150 `kernel-server --personal --bind 127.0.0.1:48681`; product binary `1d50fcc9` (docs-only delta to HEAD `625e9ccd`); `/ui/` GET 200 serves `index-CvtgBWRI.js`. dsh web on 3080. Left `:48181` untouched. |
| D02 Dual Track activation (title ≠ unknown, leave `creating`, PlanRevision axis) | pass | Same guest daemon: `draft.create` → `preview.request` → `confirm` 200. List: `state=active`, `title_summary=P14-T03 D02 titled live` (not `unknown`). Detail `plan.plan_revision_id` prefix `plan-01a072c7-cf`. Bootstrap read only from guest file; not printed. |
| J0 gate + unauthenticated fail-closed | pass | New tab `#/` showed session-gate (“Paste this daemon's bootstrap secret — not a Provider LLM API key.”). Empty Issue → `management HTTP 401; task HTTP 401. Bootstrap discarded.` Gate remained. Then same-origin one-shot fill (file deleted immediately; secret not in Git/chat/report). Header `principal://local/owner · mgmt+task`. Bare `GET /management/project/v1/list` without bearer stays 401. |
| J1 wizard click-walk ①–⑤ | pass | `#/projects/new` Dual Track ① Charter title `P14-T03 D02 wizard live` + charter → ② `确认这一环` ×3 → `确认总目标与项目触发` → `进入 ③` → ③ `创建岗位` + model `draft-bound` + six slots + `确认就位` ×3 → ④ `开始测` / `记录通过` / `通过，下一环` ×2 + `末环通过，进入 ⑤` → ⑤ `开始联调` / `核对通过` / `Request preview` minted `draft-01a072df-79d2-7252-a22d` / `preview-01a072df-7a3a-7213-b19b` / **Write Project** navigated to Today. Bundle `index-CvtgBWRI.js`. Vite not used. |
| J1 list/detail after Write | pass | List row `project-01a072df-7aa2-7920-984a-f7d77d1bef3c` **active** title **P14-T03 D02 wizard live** (not `unknown`). Detail: State `active` (not `creating`); Plan revision `plan-01a072df-7aa4-7330-895c-e11a2fbba74f`; axis collect/analyze/draft. Prior HTTP-minted `project-01a072c7-cf7c-7590-be01-a55f4671fd50` remains active titled `P14-T03 D02 titled live` / `plan-01a072c7-cf7d-7b12-95ef-64047b3ea9e6`. Two leftover `creating`/`unknown` rows from pre-T03 G1 remain honest, not this task’s live claim. |
| J10 no X/Twitter P0 hero | pass | Cursor browser → tunnel `http://127.0.0.1:48681/ui/`. CDP `twitter=false` on Today, Projects, and Knowledge. Bundle `index-CvtgBWRI.js`. |
| J18 identity | pass | `#/session` Session page; principal field `principal://local/owner`; header `principal://local/owner · mgmt+task`; Clear memory session present. Bootstrap copy is not a Provider key. |
| J19 retired routes | pass | `#/inbox`, `#/team`, `#/hitl/prev-1`, `#/home`, `#/work` each `No such route` / “This address does not exist in the Control Plane.” |
| Settings L1 + Knowledge L1 chrome | pass | Primary nav Settings and Knowledge are `role=link` (`#/settings`, `#/knowledge`). Settings h2 `Settings`; Knowledge h2 `Knowledge`. |
| Windows native chrome JOURNEY | not-run | Walk used local Cursor browser against forwarded guest `/ui/`, not Windows-native daemon chrome. |
| J1 second wizard walk (title with “titled”) | pass | Repeat ①–⑤ after session restore. Title `P14-T03 D02 wizard titled live`. Unknown-cannot-pass on ④ (`说不清。未知不能通过。` / next disabled) then pass. Draft `draft-01a072df-0624-7081-b5a8-2800bac0346d` / preview `preview-01a072df-06ac-78d1-8e31-23cba66508f5`. List `project-01a072df-8ff5-72a2-8b82-3c27f62365d7` **active**; detail Plan revision `plan-01a072df-8ff5-…`. Corroborates the first wizard walk; not a second claim. |

Unique next: ready/merge PR [#330](https://github.com/agentkernel/cognitive-os/pull/330) on the closure HEAD after required CI, then claim `P14-T04`. Do not claim T05/T06/T07/T08.

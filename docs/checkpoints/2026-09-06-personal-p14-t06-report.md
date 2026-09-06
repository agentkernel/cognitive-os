# P14-T06 Today live packets — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P14-T06` / slices `P14-T06/D01` **done** + `P14-T06/D02` **done** (this close)
- Branch: `personal/P14-T06-today-packets`
- Worktree: `D:\agent-kernel-wt-P14-T06`
- Lease: `lease/personal/P14-T06/today-packets`
- Draft PR: [#333](https://github.com/agentkernel/cognitive-os/pull/333)
- Implementation pin: Dual Track D01 then fold T04 `3012db11`; fold T05 `origin/main@d9cee39d` + D02 heartbeat `01004a49`
- Change class: `implementation-only` (daemon `/ui/` Today live packets + per-live-Project overview after T03 activation; no new authority writer; no `core/specs`; no numbered migration)
- Claim ceiling: `hypothesis`
- Product origin: daemon-served `/ui/` (`http://127.0.0.1:48681/ui/`). Vite is not the product origin.
- Evaluation routing: **OFF**
- Do not claim T07/T08 (already **done**). T01–T05 remain **done**.

## Failure-first (D01)

| ID | Negative | Surface | Observed |
|---|---|---|---|
| N1 | KPI wall (`kpi_wall: true` / success rate / weekly report) must not render as Today chrome | Dual Track Today | **fail** then **pass**: `data-kpi-wall=refused`; counts omitted |
| N2 | Unactivated titled `creating` Project must not fetch or paint live packets / overview | Dual Track Today | **pass** already (creating-only continue-create) |
| N3 | T13 empty chrome is not packet acceptance | Dual Track Today | **pass** already (empty home only-create) |
| N4 | Packets only for the first live id; second live Project’s pending preview missing / deep-linked to the first id | Dual Track Today | **fail** then **pass** |
| N5 | After activation, leftover `creating` drafts keep Today on Continue create | Dual Track Today | **fail** then **pass** |

## Incremental validation log

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-06 | D01 Dual Track Today live packets | **pass** 38/38 (`todayLivePackets` + `todayPackets` + `todayOverview` + `opcIa`); `clients/pc/web` `pnpm test` 74 files / 533 | Node jsdom | `3012db11` | failure-first then pass; required CI [33991080971](https://github.com/agentkernel/cognitive-os/actions/runs/33991080971) **SUCCESS** |
| 2026-09-06 | Fold `origin/main@a6247f09` (T04) | pass | T06 worktree | `3012db11` | last-merger kept T04 done / T05+T06 in-progress |
| 2026-09-06 | Fold `origin/main@d9cee39d` (T05 #332) + D02 lease heartbeat | pass | T06 worktree | `01004a49` | T05 done; unique Slice D02; T06/DOC-REFRAME adjacent; T05 lease only in §3.1 |
| 2026-09-06 | Exact-revision Linux build of pushed `01004a49` | pass | `DEV-LINUX-NATIVE-01` `wuz@192.168.1.2` | `01004a49` | worktree `/home/wuz/cognitiveos-personal-worktrees/p14-t06-01004a49`; `kernel-server` ELF; UI dist `index-05YCvTci.js` + `index-CjrQkJT6.css` |
| 2026-09-06 | Guest `:48681` replace (T05 released) | pass | `B01-Desktop-Linux-002` | `01004a49` | runtime `/home/hal9001/p13-main-711a5a7c/`; PID **2684337** `kernel-server --personal --bind 127.0.0.1:48681`; product UI `runtime/data/cognitiveos/ui/`; `/ui/` GET 200 serves `index-05YCvTci.js`. Left `:48181` untouched (PID 166715, `cos-current`). `cognitive dsh web` restarted on 3080. Unauth `GET /management/project/v1/list` **401**. |
| 2026-09-06 | J0 gate + unauthenticated fail-closed | pass | Cursor browser → tunnel `/ui/` | `01004a49` | Empty Issue → `management HTTP 401; task HTTP 401. Bootstrap discarded.` Gate remained. Bare list **401**. Same-origin one-shot `/ui/.boot-once` fill (file deleted immediately; secret not in Git/chat/report). Header `principal://local/owner · mgmt+task`. Status `management ready; task ready. Bootstrap discarded.` |
| 2026-09-06 | J2 Today live packets + per-Project overview | pass | guest `/ui/#/` | `01004a49` / SPA `index-05YCvTci.js` | `data-surface=today` (not `today-incomplete`). No Continue create. Honesty is T06 copy (after a live Project, Today is packets + one row per live Project). 4 live Projects: `P14-T03 D02 titled live` / `wizard live` / `wizard titled live` / `P14-T04 D02 wizard join live`. Leftover drafts honesty `opc-today-leftover-drafts`. Packet collapsed `data-collapsed=true` (“nothing pending”; overview stays). Period today `created 2 · live 4 · blocked 0`; week switch `data-period=week`. Chat cannot Approve. No KPI wall chrome (`data-kpi-wall` absent on live overview; Dual Track N1 covers `kpi_wall:true`). Vite not used. |
| 2026-09-06 | J1 titled-live regression | pass | `/ui/#/projects` + detail + `#/projects/new` | `01004a49` | List still titled live (four P14-T03/T04 rows, not `unknown`). Detail PlanRevision `plan-01a0731e-cfb5-7611-bb17-4c55852933d9` + collect/analyze/draft axis. `#/projects/new` Dual Track ①–⑤ chrome. Full ①–⑤ click-walk not repeated (T03/T04 already walked this runtime). |
| 2026-09-06 | J10 no X/Twitter P0 hero | pass | Today, Projects, Knowledge | `01004a49` | CDP `twitter=false`. Bundle `index-05YCvTci.js`. |
| 2026-09-06 | J18 identity | pass | `#/session` | `01004a49` | principal `principal://local/owner`; header `principal://local/owner · mgmt+task`; Clear memory session present. Bootstrap copy is not a Provider key. |
| 2026-09-06 | J19 retired routes | pass | `#/inbox` `#/team` `#/hitl/prev-1` `#/home` `#/work` | `01004a49` | each `No such route` / “This address does not exist in the Control Plane.” |
| 2026-09-06 | Windows native chrome JOURNEY | not-run | walk used local Cursor browser against forwarded guest `/ui/` | — | not Windows-native daemon chrome |

## D01 Dual Track mapping

- Empty home: only-create; no pending-previews / today.overview fetch; not packet acceptance.
- Titled `creating`: continue-create; no live packets.
- Titled `active` (T03 Write): Today surface + Owner title + packets + overview; not `today-incomplete`.
- Every live Project: GET `pending-previews?subject_ref=` and canvas deep-link uses that `subject_ref`.
- Leftover drafts after a live Project: `opc-today-leftover-drafts` honesty; no Continue create CTA.
- `kpi_wall: true`: `data-kpi-wall=refused`; counts strip omitted; per-live rows stay; no success rate / weekly report.
- Chat has no Approve. Confirm stays on management HTTP.

## Unique next

Ready/merge PR [#333](https://github.com/agentkernel/cognitive-os/pull/333) on the closure HEAD. After T06 close: Phase 14 Remaining = 0. Do not claim T07/T08. Do not auto-claim P6. `P7-T07` stays blocked.

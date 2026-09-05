# P14-T05 Attempt/Runs/Outputs from Project chrome — running report

- Task / slice: `P14-T05/D02` guest `/ui/` J14 (D01 Dual Track local pass)
- Lease: `lease/personal/P14-T05/attempt-runs-outputs`
- Worktree: `D:\agent-kernel-wt-P14-T05`
- Branch: `personal/P14-T05-attempt-runs`
- Draft PR: [#332](https://github.com/agentkernel/cognitive-os/pull/332)
- Implementation revision: `d72c2847` (D01 Dual Track). Fold `origin/main@a6247f09` (T04 #331) then lease-ledger CI fix `cf6d09e3`.
- Change class: `implementation-only` (Project Runs Write Attempt + whitelist management `dsh.hosted.attempt.run`; Runs/Outputs honesty; no `core/specs`; no numbered migration)
- Claim ceiling: `hypothesis`
- Product origin: daemon `/ui/` (`http://127.0.0.1:48681/ui/`) — Vite is not the product source
- Guest: `B01-Desktop-Linux-002` `hal9001@192.168.123.160` runtime `/home/hal9001/p13-main-711a5a7c/`
- Evaluation routing: **OFF**
- Do not claim T06/T07/T08. T02/T03/T04/T07/T08 remain **done**. T06 stays in-progress on PR [#333](https://github.com/agentkernel/cognitive-os/pull/333) (A8).

## Failure-first (D01)

Observed fail on Dual Track: Linux 1.0 `#/work` is not 2.0 Attempt chrome; Vite is not product origin; Write Attempt without live+seated is blocked and dispatches nothing; task-channel `attempt.run` stays false; empty ledger/CAS stays honest.

## Local development evidence (not supported CI)

- Isolated worktree from `origin/main@ed893951`; later folded `a6247f09`
- `pnpm test` in `clients/pc/web` **33/33** — `projectRuns` 8, `projectOutputs` 7, `normalize` 12, `routineRuns` 6
- No Rust/kernel files in D01; HTTP `attempt.run` already exists from P13-T02

## Units

| Unit | Result | Evidence |
|---|---|---|
| D01 Dual Track | pass | `0e4c4984` / `d72c2847`; web **33/33**; Draft PR [#332](https://github.com/agentkernel/cognitive-os/pull/332) |
| Fold `origin/main@a6247f09` (T04) | pass | merge `ed041b9c`; last-merger keeps T04 **done**, T05 **in-progress**, T06 **in-progress**; adjacent T05/T06/DOC-REFRAME lease rows, no blank line, no duplicate T05 row |
| Lease-ledger CI fix | pass | `cf6d09e3` drops `PARALLEL-LANES.md` from T05 writable paths (lease must not own the ledger) |
| Required CI at `cf6d09e3` | pass | [33991619494](https://github.com/agentkernel/cognitive-os/actions/runs/33991619494) **SUCCESS** (resolve, ubuntu, windows 20m54s, required-ci) |
| Exact-revision Linux build | pass | `wuz@192.168.1.2` worktree `/home/wuz/cognitiveos-personal-worktrees/p14-t05-cf6d09e3`; `HEAD=cf6d09e3`; dirty=0; `kernel-server` ELF 42795608; UI dist `index-BxtJu4NK.js` |
| Guest daemon replace on 48681 | pass | PID 2671617 `kernel-server --personal --bind 127.0.0.1:48681`; product binary `cf6d09e3`; `/ui/` GET 200 serves `index-BxtJu4NK.js`. `cognitive dsh web` restarted on 3080 (PID 2672166). Left `:48181` untouched (PID 166715, `cos-current`). |
| J0 gate + unauthenticated fail-closed | pass | Empty Issue → `management HTTP 401; task HTTP 401. Bootstrap discarded.` Gate remained. Bare `GET /management/project/v1/list` without bearer **401**. Empty JSON `/local/session` **400**. Same-origin one-shot `/ui/.boot-once` fill (file deleted immediately; secret not in Git/chat/report). Header `principal://local/owner · mgmt+task`. |
| J14 Project chrome Write Attempt | pass | `#/projects/project-01a0731e-cfb2-72b2-9025-5e8372f45dc1/runs` (T04 Dual Track titled **P14-T04 D02 wizard join live**, PlanRevision `plan-01a0731e-cfb5-7611-bb17-4c55852933d9`). Pending roster blocked Write Attempt (`data-write-attempt` disabled, nothing dispatched). After management `seat.request` + `seat.confirm` with `model_binding=draft-bound` on the three T04 employees, Write Attempt posted management `dsh.hosted.attempt.run` and Attempt history showed `dshattempt-01a0736b-d14f-7f02-9b16-d865702245da` for seated `employee-01a07320-5d58-…`. URL stayed `#/projects/…/runs`, not `#/work`. Bundle `index-BxtJu4NK.js`. Vite not used. |
| J14 Runs/Outputs honesty | pass | Runs: daemon occurrence ledger empty (“No occurrence recorded”) + real Attempt history row. Outputs: **Project outputs: no openable artifact yet** (honest empty CAS; not a fake gallery). Honesty copy: daemon `/ui/`, not Linux 1.0 `#/work`. |
| J1 regression | pass | Projects list still titled live: `P14-T03 D02 titled live` / `P14-T03 D02 wizard live` / `P14-T04 D02 wizard join live` (not `unknown`). Detail PlanRevision + collect/analyze/draft axis. `#/projects/new` Dual Track ①–⑤ chrome. Full ①–⑤ click-walk not repeated this round (T04 already walked it on this runtime; T05 did not change the wizard). |
| J10 no X/Twitter P0 hero | pass | CDP `twitter=false` on Today, Projects, Knowledge. Bundle `index-BxtJu4NK.js`. |
| J18 identity | pass | `#/session` Session page; principal `principal://local/owner`; header `principal://local/owner · mgmt+task`; Clear memory session present. Bootstrap copy is not a Provider key. |
| J19 retired routes | pass | `#/inbox`, `#/team`, `#/hitl/prev-1`, `#/home`, `#/work` each `No such route` / “This address does not exist in the Control Plane.” |
| Windows native chrome JOURNEY | not-run | Walk used local Cursor browser against forwarded guest `/ui/`, not Windows-native daemon chrome. |

## Unique next

Ready/merge PR [#332](https://github.com/agentkernel/cognitive-os/pull/332) on this closure HEAD. Unique next after T05 close: `P14-T06/D02` on the same guest with the pushed T06 HEAD (PR [#333](https://github.com/agentkernel/cognitive-os/pull/333) still OPEN Draft at `3012db11`; sibling D02 not in flight). Do not claim T07/T08.

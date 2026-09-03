# P13-T05 Routine/Trigger arming, occurrence ledger, `runs` + Today — running report

- Task: `P13-T05` / slices `P13-T05/D01` → `P13-T05/D02`
- Change class: `implementation-only` (Routine arming + scheduler-driven occurrence ledger in `cognitive-store` / kernel-server; `runs` + Today projections in `clients/pc/web`; no `core/specs`, no Lane-CTR, no new first-level chrome, no second scheduler; additive authority migration)
- Product: CognitiveOS Personal 2.0.0
- Lease: `lease/personal/P13-T05/routine-runs`
- Branch: `personal/P13-T05-routine-runs` (worktree `D:\agent-kernel-wt-p13-t05`; original `d:\agent-kernel` untouched, A8 protected)
- Base: `origin/main@3d66d66d` (P13-T03 lease closure)
- Claim ceiling: `hypothesis` (A7: local / CI / Linux native is not Gate / release / Profile; Linux native evidence closes "implementation exists" only; clock / sleep / restart host E2E stays `not-run` until `P13-T13`)
- Evaluation routing: **OFF**

## Unique next action

D01: write the failure-first negatives (second scheduler, overlap, silently dropped occurrence, checkpoint treated as completion, arming before G2, running-prompt injection), observe them fail on CI / Linux, then wire arming → daemon scheduler tick → Intent → hosted Attempt → occurrence ledger.

## Scope decision recorded at claim (PlanRevision-apply gap)

P13-T02 found that no product HTTP / CLI path applies a PlanRevision, records a stage test, or reaches G2 acceptance (`roster.register` needs `plan_revision_id`; `apply_plan_revision`, `record_stage_test` and the `acceptance` preview subject are store-only). Arming after G2 needs all three facts (an `active` accepted Project, a current plan stage, a seated responsible Member). Decision: **not widened into this card**. Arming fails closed before G2 (`ROUTINE_ARM_BEFORE_G2`), unit tests and the live E2E seed those facts as a fixture exactly as P13-T02 did (scratch seed run before the daemon starts), and the gap stays owned by `P13-T04` (stage test / last-ring acceptance) and `P13-T06` (`@manager` plan revision). Recorded in PROGRESS Layer 2 and the plan.md card.

## Failure-first (D01)

| ID | Negative | Surface |
|---|---|---|
| N1 | second scheduler refused: the only dispatcher of `task://personal/routine/*` rows is the daemon scheduler tick; a second lease holder is fenced (`acquire_eligible_lease` epoch / owner CAS) and an occurrence can never be dispatched twice | store + kernel-server tick |
| N2 | overlap refused: one Routine never has two `active` occurrences; a schedule firing while one is active queues (latest only) and never spawns a second Attempt | store + tick |
| N3 | silent drop forbidden: a schedule firing while the host is paused / offline lands as `missed` with a reason; a manual trigger on an un-armed Routine lands as `missed` (`not-armed`), never vanishes; coalesced occurrences keep `coalesced_by` | store + tick |
| N4 | checkpoint ≠ completion: `record_checkpoint(complete = true)` refused; an occurrence reaches `attempted` only through a daemon-observed Attempt terminal, and `completion_claimed` stays false | store |
| N5 | arming before G2 refused (`creating` Project → `ROUTINE_ARM_BEFORE_G2`); arming with an unseated / wrong-slot Member refused; stale Routine revision refused | store + HTTP |
| N6 | running-prompt injection refused: a new instruction revision never changes the `context_digest` of the active occurrence's Attempt; `continue` applies from the next occurrence, `pause` stops new occurrences, `restart` queues a new-revision occurrence behind the active one | store + tick |
| N7 | process exit ≠ cancel / complete: Attempt terminal `exited/0`, `failed`, `timed-out`, `unknown-outcome` all land as occurrence outcome facts with `verification_status = not-run` | store + tick |
| N8 | task-channel aliases 403; secret-shaped declaration refused; no fake Start: manual trigger is the existing Intent path `routine.trigger` | HTTP |

## Incremental validation log (TEST-REPORT-INCREMENTAL-01)

Units are appended **immediately** after each finishes. `not-run` is never pass.

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-03 | Worktree `D:\agent-kernel-wt-p13-t05` created from `origin/main@3d66d66d`; lease row + PROGRESS / formal plan / plan.md → `P13-T05` in-progress, `P13-T05/D01` in-progress; report skeleton | recorded | docs-only (`DEV-WIN-GNU-01`) | worktree, uncommitted | `git worktree list` / `git branch --list "personal/P13-T05*"` showed no prior branch or worktree |

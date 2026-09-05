# P14-T05 Attempt/Runs/Outputs from Project chrome — running report

- Task / slice: `P14-T05/D01` Dual Track (D02 guest `/ui/` J14 **blocked** this round)
- Lease: `lease/personal/P14-T05/attempt-runs-outputs`
- Worktree: `D:\agent-kernel-wt-P14-T05` (isolated from primary T04 checkout)
- Branch: `personal/P14-T05-attempt-runs` from `origin/main@ed893951` (PR #330 / T03 merged)
- Draft PR: pending this delivery
- Change class: `implementation-only` (Project Runs Write Attempt + whitelist management `dsh.hosted.attempt.run`; Runs/Outputs honesty; no `core/specs`; no numbered migration)
- Claim ceiling: `hypothesis`
- Product origin: daemon `/ui/` — Vite is not the product source
- Evaluation routing: **OFF**
- Do not claim T04/T06/T07/T08. T03/T07/T08 remain **done**. Do not replace guest `:48681` (T04/D02 owns the guest this round).

## Isolation

T04 sibling Draft PR [#331](https://github.com/agentkernel/cognitive-os/pull/331) owns member-join paths. This lease does **not** include those files, `docs/plan/PROGRESS.md`, `PERSONAL-DEVELOPMENT-PLAN.md`, `plan.md`, or handbook HTTP pages. `PARALLEL-LANES.md` is coordination-only (never in writable paths). Current snapshot on this branch adds a P14-T05 row so CI can name the active lease; T04 has a dirty lock on those plan files in the primary worktree — last-merger must keep both T04 and T05 facts. Formal plan Layer 1 still lists P14-T05 `not-started` because that file is T04-owned; status for this slice lives here and in `PROGRESS.md` on this branch.

## Units

| Unit | Result | Evidence |
|---|---|---|
| Claim + isolated worktree | pass | `git worktree add -b personal/P14-T05-attempt-runs D:\agent-kernel-wt-P14-T05 origin/main` at `ed893951`; `rustup override set 1.97.1-x86_64-pc-windows-msvc`; `rustc -vV` host `x86_64-pc-windows-msvc`; lease row adjacent to product-prototype-docs (no blank line) |
| Failure-first Dual Track | pass | Tests written for `#/work` not 2.0, Vite not product origin, blocked Write Attempt without live+seated, task-channel `attempt.run` stays false, empty ledger/CAS honest. After Write Attempt + management whitelist: `pnpm test` in `clients/pc/web` **33/33** across `projectRuns.test.tsx` (8), `projectOutputs.test.tsx` (7), `normalize.test.ts` (12), `routineRuns.test.ts` (6) |
| Behavior change | pass | Project Runs: **Write Attempt** POSTs management `dsh.hosted.attempt.run` (`wait:false`) when Project `active` and roster has a seated Member; otherwise `data-write-attempt=blocked` and nothing is dispatched. `#/work` stays retired (`No such route`). Honesty: daemon `/ui/`, Vite is not product origin. Outputs empty CAS stays “no openable artifact yet”, not a fake gallery |
| Local MSVC cargo | not-run | No Rust/kernel files in this slice; HTTP `attempt.run` already exists from P13-T02. Toolchain pin only: `rustc 1.97.1` host `x86_64-pc-windows-msvc` |
| Guest `/ui/` J14 (`P14-T05/D02`) | blocked | T04/D02 owns `B01-Desktop-Linux-002` `:48681` this round. Do not deploy/replace the guest daemon. Unique next after D01: D02 when that slot is free |
| Ready/merge | not-run | D01 does not ready/merge; D02 JOURNEY remains |

Unique next: keep Draft PR; do not ready/merge; `P14-T05/D02` guest `/ui/` J14 + `JOURNEY-BROWSER-SYNC-01` when T04 D02 releases `:48681`.

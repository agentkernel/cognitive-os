# P14-T06 Today live packets — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. Append each finished unit immediately. `not-run` is never pass. Claim ceiling `hypothesis`. A7: local/CI is not Gate.

- Task: `P14-T06` / slice `P14-T06/D01`
- Branch: `personal/P14-T06-today-packets`
- Worktree: `D:\agent-kernel-wt-P14-T06` from `origin/main@ed893951` (PR [#330](https://github.com/agentkernel/cognitive-os/pull/330) T03 merge). Primary checkout `D:\agent-kernel` (`personal/P14-T04-member-join`) was not edited (A8).
- Lease: `lease/personal/P14-T06/today-packets`
- Change class: `implementation-only` (daemon `/ui/` Today live packets + per-live-Project overview after T03 activation; no new authority writer; no `core/specs`; no numbered migration)
- Claim ceiling: `hypothesis`
- Product origin: daemon-served `/ui/`. Vite is not the product origin.
- Evaluation routing: **OFF**
- Do not claim T04/T05/T07/T08. Do not deploy/replace guest `:48681`. `P14-T06/D02` is not this turn.

## Unique next action

`P14-T06/D02` guest `/ui/` J2 + `JOURNEY-BROWSER-SYNC-01` **after T05 releases guest `:48681`**. Do not deploy or replace `:48681`. Do not start J2 this turn. Keep Draft PR [#333](https://github.com/agentkernel/cognitive-os/pull/333).

## Wait-gate / guest lock

T04 last-merger on `origin/main@a6247f09` (PR [#331](https://github.com/agentkernel/cognitive-os/pull/331)) already recorded T04 **done**, T05 **in-progress** (PR [#332](https://github.com/agentkernel/cognitive-os/pull/332)), T06 **in-progress** (PR [#333](https://github.com/agentkernel/cognitive-os/pull/333)). Plan-file lock is lifted. This fold keeps those three facts and the T05 lease row (A8).

T05 is using guest `:48681` for T05/D02. T06 must not deploy, replace, or walk that daemon. `P14-T06/D02` stays `ready` until T05 releases `:48681`.

## Isolation

Writable paths: `clients/pc/web/src/views/opc/TodayPage.tsx`; `clients/pc/web/src/views/opc/todayLiveReads.ts`; `clients/pc/web/src/views/opc/todayLivePackets.test.tsx`; `clients/pc/web/src/views/opc/todayPackets.test.tsx`; `docs/checkpoints/2026-09-06-personal-p14-t06-report.md`.

Not taken: T04 member-join store/HTTP/AddMember; T05 Project chrome Attempt / Runs / Outputs / hosted-attempt launch; `loadOpcReads.ts` (read-only import of `loadTodayOverview`); guest `:48681`. Sibling worktree `D:\agent-kernel-wt-P14-T05` exists on `personal/P14-T05-attempt-runs` at `ed893951` (no T05 lease on `origin/main` at claim time).

## Failure-first (D01)

| ID | Negative | Surface | Observed on `ed893951` before change |
|---|---|---|---|
| N1 | KPI wall (`kpi_wall: true` / success rate / weekly report) must not render as Today chrome | Dual Track Today | **fail** then **pass**: no `data-kpi-wall=refused`; counts still painted |
| N2 | Unactivated titled `creating` Project must not fetch or paint live packets / overview | Dual Track Today | **pass** already (creating-only continue-create) |
| N3 | T13 empty chrome is not packet acceptance | Dual Track Today | **pass** already (empty home only-create) |
| N4 | Packets only for the first live id; second live Project’s pending preview missing / deep-linked to the first id | Dual Track Today | **fail** then **pass** |
| N5 | After activation, leftover `creating` drafts keep Today on Continue create | Dual Track Today | **fail** then **pass** |

## Incremental validation log

| Time | Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|---|
| 2026-09-06 | Worktree `D:\agent-kernel-wt-P14-T06` from `origin/main@ed893951`; `rustup override set 1.97.1-x86_64-pc-windows-msvc` (`host: x86_64-pc-windows-msvc`); lease row claimed; Dual Track tests written first | recorded | `DEV-WINDOWS-NATIVE-OPC-01` / Node | worktree | Primary T04 worktree not edited |
| 2026-09-06 | Failure-first Dual Track `todayLivePackets` on current Today | **fail** 3 / **pass** 3 | vitest jsdom | `ed893951` + tests-only | N2/N3/titled-live single-project **pass**; N1/N4/N5 **fail** as predicted |
| 2026-09-06 | Behavior: per-live `pending-previews` merge; leftover drafts not Continue create; KPI wall refused; HITL_KEY mirror for assistant rail | recorded | worktree | uncommitted | `todayLiveReads.ts`; TodayPage; P12 mixed test updated |
| 2026-09-06 | Dual Track `todayLivePackets` + `todayPackets` + `todayOverview` + `opcIa` | **pass** 38/38 | vitest | worktree | after HITL_KEY mirror for `opc-rail-hitl` |
| 2026-09-06 | `clients/pc/web` `pnpm test` | **pass** 74 files / 533 tests | Node | worktree | development evidence only |
| 2026-09-06 | `clients/pc/web` `pnpm build` | **pass** | tsc + vite | worktree | first tsc fail TS2345 loading fallback; typed `Projection[]`; bundle `index-SUbyu6TL.js` |
| 2026-09-06 | Guest `/ui/` J2 / `JOURNEY-BROWSER-SYNC-01` / replace `:48681` | **not-run** | T05 owns guest for D02 | — | T06 must not deploy/replace `:48681` |
| 2026-09-06 | Local MSVC `cargo` | **not-run** | no Rust change; Dual Track TS is the registered D01 surface | — | override set; not used as supported validation |
| 2026-09-06 | `PROGRESS.md` / formal plan / `plan.md` snapshot | **blocked** then **lifted** | T04 merged PR [#331](https://github.com/agentkernel/cognitive-os/pull/331) at `main@a6247f09` already recorded T04 done / T05+T06 in-progress | `a6247f09` | this fold keeps those facts; T05 lease row retained |
| 2026-09-06 | Fold `origin/main@a6247f09` (T04) into `personal/P14-T06-today-packets`; keep T05 lease adjacent; D02 blocked on T05 guest | recorded | T06 worktree only | fold | no J2; Draft PR [#333](https://github.com/agentkernel/cognitive-os/pull/333) |

## D01 Dual Track mapping

- Empty home: only-create; no pending-previews / today.overview fetch; not packet acceptance.
- Titled `creating`: continue-create; no live packets.
- Titled `active` (T03 Write): Today surface + Owner title + packets + overview; not `today-incomplete`.
- Every live Project: GET `pending-previews?subject_ref=` and canvas deep-link uses that `subject_ref`.
- Leftover drafts after a live Project: `opc-today-leftover-drafts` honesty; no Continue create CTA.
- `kpi_wall: true`: `data-kpi-wall=refused`; counts strip omitted; per-live rows stay; no success rate / weekly report.
- Chat has no Approve. Confirm stays on management HTTP.

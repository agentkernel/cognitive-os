# P14-T07 Settings L1 + IA — running validation report

- Task: `P14-T07` / slices `P14-T07/D01` then `P14-T07/D02`
- Lease: `lease/personal/P14-T07/settings-ia`
- Branch: `personal/P14-T07-settings-l1` (worktree `D:\agent-kernel-p14-t07`)
- Claim ceiling: `hypothesis`. Not Gate / release / Profile / B01 / EVAL-016 revival.
- Product origin: daemon `/ui/`. Vite is not the product origin. dsh is not 小白 chrome.
- Oracle: EVAL-016 J6 / J8 / J12 / J20; `JOURNEY-BROWSER-SYNC-01`

## Wait gate

- `lease/personal/DOC-P14-GAP-CLOSE/plan-registration` was **closed** → PARALLEL-LANES §3.1.
- Rebased this task branch onto `e14bc7a7` (`docs(DOC-P14-GAP-CLOSE): persist Phase 14 registration and close the docs lease`).
- Evaluation routing OFF. Did not claim T02/T03/T04 (Worker 1) or T08/T05 (Worker 3).
- Did not write `D:\agent-kernel` (Worker 1 dirty main / DOC worktree). Did not write `personal/docs/product/**` (`DOC-PERSONAL-2.0-OPC-REFRAME` still active).

## Slice status

| Slice | Status | Notes |
|---|---|---|
| `P14-T07/D01` | `in-progress` | Settings L1 `role=link`; default 9×9 unmounted; Linux 1.0 Home/Work/Agents/Providers hashes 404; palette Escape capture; `/settings/model-connections` hub |
| `P14-T07/D02` | `ready` | exact-revision guest `/ui/` J6/J8/J12/J20 + `JOURNEY-BROWSER-SYNC-01` |

## Validation log (TEST-REPORT-INCREMENTAL-01)

| When | Unit | Result |
|---|---|---|
| 2026-09-05 | Dual Track `pnpm test` in `clients/pc/web` (MSVC not required; TS only) | **pass** 72 files / 516 tests |
| 2026-09-05 | `ownerChromeIa.test.tsx` (failure-first then green) | **pass**: Settings L1 link; 0× `data-state-lab-cell` by default; `#/home` `#/work` `#/work/new` `#/agents` `#/providers` `#/bindings` `#/tasks` = No such route; palette does not advertise retired hashes; `#/settings/model-connections` SecretStore form; Escape closes palette |
| 2026-09-05 | leftover Home/Work/Agents/Providers suites | **pass** via test-only `LinuxLegacyApp` (not product chrome) |
| 2026-09-05 | required CI | `not-run` until push |
| 2026-09-05 | guest `/ui/` J6/J8/J12/J20 + `JOURNEY-BROWSER-SYNC-01` | `not-run` (D02) |

## Unique next

Push Draft PR, then D02 on `B01-Desktop-Linux-002` at the pushed revision: J6/J8/J12/J20 plus regression J0/J10/J18/J19.

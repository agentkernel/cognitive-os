# P14-T07 Settings L1 + IA — running validation report

- Task: `P14-T07` / slices `P14-T07/D01` then `P14-T07/D02`
- Lease: `lease/personal/P14-T07/settings-ia` **closed** → PARALLEL-LANES §3.1
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
| `P14-T07/D01` | `done` | Dual Track 72/516. Settings L1; 9×9 unmounted; Linux 1.0 hashes 404; palette Escape capture; `#/settings/model-connections` SecretStore hub |
| `P14-T07/D02` | `done` | guest `/ui/` J6/J8/J12/J20 + `JOURNEY-BROWSER-SYNC-01` J0/J10/J18/J19 **pass** at SPA `634da855` on `B01-Desktop-Linux-002` `127.0.0.1:48681`; required CI [33975415810](https://github.com/agentkernel/cognitive-os/actions/runs/33975415810) **SUCCESS** at `7264b68d` |

## Validation log (TEST-REPORT-INCREMENTAL-01)

| When | Unit | Result |
|---|---|---|
| 2026-09-05 | Dual Track `pnpm test` in `clients/pc/web` (MSVC not required; TS only) | **pass** 72 files / 516 tests |
| 2026-09-05 | `ownerChromeIa.test.tsx` (failure-first then green) | **pass**: Settings L1 link; 0× `data-state-lab-cell` by default; `#/home` `#/work` `#/work/new` `#/agents` `#/providers` `#/bindings` `#/tasks` = No such route; palette does not advertise retired hashes; `#/settings/model-connections` SecretStore form; Escape closes palette |
| 2026-09-05 | leftover Home/Work/Agents/Providers suites | **pass** via test-only `LinuxLegacyApp` (not product chrome) |
| 2026-09-05 | required CI run [33974591057](https://github.com/agentkernel/cognitive-os/actions/runs/33974591057) at `634da855` | superseded by fold CI |
| 2026-09-05 | required CI run [33975415810](https://github.com/agentkernel/cognitive-os/actions/runs/33975415810) at `7264b68d` | **SUCCESS** — resolve 2s; ubuntu 4m29s; windows 17m17s; required-ci 2s |
| 2026-09-05 | guest `/ui/` J0 gate | **pass** — bootstrap secret into password field; not a Provider key; session issued; Vite is not the product origin |
| 2026-09-05 | guest `/ui/` J8 Settings | **pass** — Primary `role=link` Settings current; `#/settings` and `#/settings/model-connections` both show Model Connections; `Hand key to SecretStore` disabled with empty key; 0 fake Connect buttons; 0 `#/home` `#/work` `#/agents` `#/providers` leftover links |
| 2026-09-05 | guest `/ui/` J6/J12 chrome | **pass** — default Settings `data-state-lab-cell` count **0**; open Advanced state-lab → **81** cells; close → **0** (unmounted). HITL pending preview **not-run** (no ApprovalPreview after EVAL-016 creating Project; T03/T04) |
| 2026-09-05 | guest `/ui/` J19 + retired hashes | **pass** — `#/home` `#/work` `#/work/new` `#/agents` `#/providers` `#/inbox` `#/team` all **No such route** |
| 2026-09-05 | guest `/ui/` J20 + palette intercept | **pass** — skip link present; Settings is L1 link; palette destinations are Today/Projects/Knowledge/Settings/Model Connections/Activity (no Home/Work/Agents/Providers); Escape closes dialog (`role=dialog` gone); Import to Vault click then reached the Knowledge button |
| 2026-09-05 | `JOURNEY-BROWSER-SYNC-01` regression J10 / J18 | **pass** — no X/Twitter P0 in L1; Session link + principal remain |
| 2026-09-05 | dsh `http://127.0.0.1:3080/` | HTTP **200** (DSH Local Build). Not 小白 chrome. |

## Unique next

Ready/merge PR [#327](https://github.com/agentkernel/cognitive-os/pull/327) after required CI on the closure HEAD. Formal plan three-column stays `not-started` until last merger (T08 listed `PERSONAL-DEVELOPMENT-PLAN.md`). Do not claim T02/T03/T04/T08/T05. T03 is not done — do not claim T06.

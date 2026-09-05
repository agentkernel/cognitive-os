# P14-T07 Settings L1 + IA — closure

- Task: `P14-T07` **done** / slices `P14-T07/D01` **done** + `P14-T07/D02` **done**
- Change class: `implementation-only` (Owner chrome IA: Settings L1, unmount default 9×9 state-lab, retire Dual Track Linux 1.0 hashes from product App / leftover lists / command palette; `#/settings/model-connections` hub). No `core/specs`. A6 negatives not weakened.
- Lease: `lease/personal/P14-T07/settings-ia` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P14-T07-settings-l1` (worktree `D:\agent-kernel-p14-t07`) → PR [#327](https://github.com/agentkernel/cognitive-os/pull/327)
- Implementation HEAD: `634da855` (SPA Dual Track). Required CI on fold `7264b68d` (merge `origin/main@adb20828` / #326): [33975415810](https://github.com/agentkernel/cognitive-os/actions/runs/33975415810) **SUCCESS** (resolve 2s, ubuntu 4m29s, windows 17m17s, required-ci 2s)
- Guest `/ui/`: `B01-Desktop-Linux-002` `127.0.0.1:48681`. SPA `634da855` served on kernel pin `711a5a7c` (Dual Track-only; kernel binary unchanged). Product origin = daemon `/ui/`. Vite is not the product origin. dsh `3080` is not 小白 chrome.
- Running report: [P14-T07 report](2026-09-05-personal-p14-t07-report.md)
- Claim ceiling: `hypothesis`. Not Gate / release / Profile / B01 / EVAL-016 revival.
- Formal plan three-column / Phase 14 counts / `personal-trace.yaml`: **last merger** (`P14-T08` listed `PERSONAL-DEVELOPMENT-PLAN.md` + `plan.md`). This delivery does not overlap that lease.

## 1. Acceptance mapping

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| Settings is a real L1 `role=link` | `PRIMARY_NAV` includes Settings `NavLink`; side-foot duplicate removed | `ownerChromeIa.test.tsx` | Dual Track; guest `/ui/` snapshot J8/J20 |
| Default Owner chrome hides 9×9 state-lab (not in a11y tree) | Settings `<details>` mounts `StateLabGrid` only when `open` | default 0× `[data-state-lab-cell]`; open 81; close unmounts to 0 | Dual Track; guest `/ui/` J6/J12 |
| Linux 1.0 `#/home` `#/work` `#/work/new` `#/agents` `#/providers` not Owner chrome | product `AppRoutes` 404 (same as `#/inbox`); leftover lists and palette do not advertise them | hashes render **No such route**; palette destinations are Today/Projects/Knowledge/Settings/Model Connections/Activity | Dual Track; guest `/ui/` J8/J19/J20 |
| `/settings/model-connections` or hub reachable; SecretStore (P13-T08) | same `SettingsPage`; button **Hand key to SecretStore**; no fake Connect | empty key keeps submit disabled; 0 Connect buttons; no raw secret in DOM | Dual Track; guest `/ui/` J8 |
| Palette intercept (J5/J20) | Escape capture + `stopImmediatePropagation` | Escape closes `role=dialog`; Import to Vault click reaches Knowledge button | Dual Track; guest `/ui/` J20 |
| Failure-first Dual Track before behavior | `ownerChromeIa.test.tsx` written red then green | retired hashes / missing L1 / dump 9×9 | Dual Track 72 files / 516 tests |
| `JOURNEY-BROWSER-SYNC-01` | guest `/ui/` J6/J8/J12/J20 + regression J0/J10/J18/J19 | HITL pending preview **not-run** (T03/T04; no ApprovalPreview after EVAL-016 creating Project). No already-closed Phase 14 implementation journeys besides T01 docs-only. | [report](2026-09-05-personal-p14-t07-report.md) |

## 2. Non-claims

No fake Connect. No raw secret in Git/chat/DOM/report. state-lab is not L1. Linux 1.0 hashes are not 2.0 destinations. Vite is not the product origin. dsh is not merged into `/ui/`. A6 leftover Dual Track suites remain via test-only `LinuxLegacyApp`. HITL Confirm / Routine miss / Assistant create-conversation stay on T03/T04/T05/T06. Knowledge ingest is T08. Create wizard is T02. Formal plan Layer 1 three-column still `not-started` until last merger.

## 3. Unique next

Ready/merge PR [#327](https://github.com/agentkernel/cognitive-os/pull/327) after required CI on this closure HEAD. Close lease (this commit). Do not claim T02/T03/T04 (Worker 1) or T08/T05 (Worker 3). Claim T06 only if T03 is already `done` (it is not). Evaluation routing OFF.

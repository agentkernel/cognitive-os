# P13-T12 visual spec + a11y / visual qualification — D02 / task closure

- Task: `P13-T12` **done** / slice `P13-T12/D02` **done** (D01 already merged PR [#308](https://github.com/agentkernel/cognitive-os/pull/308))
- Change class: `implementation-only` documentation (judgement sheet + report + plan closure; no product CSS/IA/canvas/`clients/pc/web`/`personal/crates` edit)
- Lease: `lease/personal/P13-T12/visual-qualification` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: Draft PR [#322](https://github.com/agentkernel/cognitive-os/pull/322), branch `personal/P13-T12-D02-visual-qualification`, worktree `D:\agent-kernel-wt-P13-T12-D02`
- Product `/ui/` evidence revision: `c8691923cd3988f0ffee9123752e073480aea5e9` (`DEV-LINUX-NATIVE-01` guest daemon `127.0.0.1:48786` + host Chrome 151 CDP/dump-dom)
- Fold HEAD (to-merge until this closure lands): `bba4aa47` (`origin/main@8e92410d` / T11 closed folded in)
- Required CI on fold HEAD: [33951377929](https://github.com/agentkernel/cognitive-os/actions/runs/33951377929) **SUCCESS** at `bba4aa47` (resolve 2s, ubuntu 4m32s, windows 14m20s, required-ci 3s)
- Running report: [2026-09-05-personal-p13-t12-d02-report.md](2026-09-05-personal-p13-t12-d02-report.md)
- Claim ceiling: `hypothesis`. Host dump-dom / CDP against Linux guest `/ui/` is implementation evidence only — not Windows native chrome, not Gate, release, or Profile.

## 1. Acceptance mapping (formal plan P13-T12 card + `P13-T12/D02`)

| Acceptance item | Recorded? | Evidence |
|---|---|---|
| D01 visual spec + frozen-v9 module-by-module `/ui/` comparison checklist | **done** (prior) | PR [#308](https://github.com/agentkernel/cognitive-os/pull/308) at `main@3680b742`; [D01 closure](2026-09-03-personal-p13-t12-d01-visual-spec-closure.md) |
| State Lab 九态 × 九表面 on exact-revision daemon `/ui/`; each cell pass/fail/not-run | **yes** — 81/81 `fail` | Grid A: Settings → Advanced mounts 81 shared-widget cells, not real surface layouts (spec §9.2). Honest fail, not patched. Report § counters + cells |
| Keyboard reachability and focus restoration | **yes** — 7 pass / 5 fail / 10 partial / 35 not-run | Grid B. K1 fail (brand `h1` vs space-title `h2`, spec §13-m). Members/runs/outputs/hitl `not-run` (no live Project on disposable runtime) |
| 200% and narrow three-column horizontal scroll (columns do not stack) | **yes** — 5 pass / 15 fail / 16 not-run | Grid C. V1 1440@100% pass; V2/V3/V4 fail on executable surfaces because `app.css` ≤1279 px stacks (spec §13-a). No CSS patch in this lease |
| light / dark / high-contrast host-theme contrast | **yes** — 18 pass / 22 not-run | Grid D. L/D/HC pass on executable surfaces (0 on-screen text pairs under 4.5:1). FC / Windows High Contrast `not-run` until `P13-T13` |
| NVDA key paths | **yes** — 10/10 `not-run` | Grid E. NVDA not installed on `DEV-WIN-GNU-01`; no invented environment ID |
| 19 table-A module judgements | **yes** — 7 pass / 1 fail / 1 partial / 10 not-run | `M-STATE` fail (shared widgets). Create steps ②–⑤, live Project, Today packets `not-run` (disposable runtime) |
| Windows native chrome | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` not provisioned; owner skip-Windows-host cells (`P13-T13`) |

Formal-plan 关闭门: D02 九态 × 九表面、键盘/焦点、200%/窄窗、host-theme、NVDA **每格记账** — **true**. Fail and `not-run` are valid accounting. Did not skip a cell as pass. Did not treat canvas/Vite as product. Did not patch State Lab into a fake static diagram. Did not write rendered Linux review as Windows native qualification.

Drift negatives from the card, all held: canvas 截图当验收 (not used); 跳过格写 pass (not done); 改 IA (not done); Vite 当产品源 (not used); 假 State Lab 静态图 (State Lab recorded `fail`); 把 rendered review 写成 Windows native 资格 (explicitly not).

## 2. Validation summary

| Environment | Result |
|---|---|
| `DEV-LINUX-NATIVE-01` + host Chrome 151 | Rendered `/ui/` grids executed at `c8691923`. Counters in the running report |
| `DEV-WIN-GNU-01` | Chrome present; NVDA absent → Grid E `not-run`. No GNU cargo linking |
| Fold HEAD `bba4aa47` required CI | [33951377929](https://github.com/agentkernel/cognitive-os/actions/runs/33951377929) **SUCCESS** (ubuntu + windows + required-ci) |
| `DEV-WINDOWS-NATIVE-OPC-01` | **not-run** until `P13-T13` |
| Local Windows-host E2E / P13-T13 | **not claimed** (owner skip-Windows-host tests) |

## 3. Non-claims

Not Gate, release, Profile, B01, Agent-benefit, or `P11-T15`. Not a CSS/IA fix for spec §13-a or §13-m. Canvas v9 ≠ product. Vite ≠ product origin. `PERS-PR-052` stays `not-run` until `P13-T13` provisions `DEV-WINDOWS-NATIVE-OPC-01` and backfills native E2E. Honest fail/not-run cells were not “fixed” into passes.

## 4. Unique next

`P13-T13` remains **excluded / not claimed** (owner 2026-09-04 Windows-host test cells; `P13-T13/D01` host blocker recorded). Do not claim `P11-T15`. Phase 13 Remaining executable work is T13 only.

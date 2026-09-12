# P15-T06 remaining layout tokens 176/22/44/~1100 — closure

- Task: `P15-T06` **done** / slices `P15-T06/D01` **done** + `P15-T06/D02` **done**
- Change class: `implementation-only` (remaining v9 shell tokens + Owner honesty demotion). T01 `--cp-shell-min-width: 1100px` not reverted. No `core/specs`. No numbered migration. Canvas v9 file not overwritten. AXIOMS.md not rewritten.
- Lease: `lease/personal/P15-T06/layout-tokens` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P15-T06-layout-tokens` → Draft PR [#341](https://github.com/agentkernel/cognitive-os/pull/341) (ready/merge after this HEAD's required CI)
- Dual Track / guest pin: `6214aa3e` walk
- Guest `/ui/` pin: `6214aa3e` / SPA `index-uL34SeNM.js` + `index-w6MOHVWe.css`
- Running report: [P15-T06 report](2026-09-06-personal-p15-t06-report.md)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI / guest `/ui/` close "remaining v9 layout tokens" only). Not Gate / release / Profile / B01. Windows native daemon chrome JOURNEY **not-run**.
- Evaluation routing: **OFF**. Phase 15 Remaining = 0 after this close. Do not merge leftover SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334). Do not auto-claim P6.

## 1. Acceptance mapping (formal plan P15-T06 card + D01/D02)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| 176px nav / 22px titles / 44px hits / ~1100px three-column | `--cp-nav-width: 176px`; `--cp-main-min: 576px`; `--cp-rail-width: 348px`; `--cp-target-min: 44px`; `--cp-size-title1: 1.375rem`; `--cp-shell-min-width: 1100px` kept; `.cp-shell` token grid | Dual Track N2 stacked `232px`/`200px` columns; T01 min-width revert | N2 **fail** then **pass**; guest J0 computed `176px 756px 348px`, title `22px`, side `176px`, rail `348px`, `min-width: 1100px` |
| Remaining Owner honesty secondary `details` | `placement="secondary"` on Assistant rail, group chat, lifecycle, member, add-member, unused authority lead | Dual Track N3 primary rail honesty | N3 **fail** then **pass**; guest J0 `primaryHonesty=0` |
| Unauthenticated fail-closed; no fake Activate; no Twitter/X P0; Vite not product | SessionGate; 0 Activate; twitter=false; daemon `/ui/` only | Dual Track N1; stacked columns; Twitter P0; Vite-as-product | N1 **pass**; guest J0 Session denied; walk 6/6 twitter=false |
| `JOURNEY-BROWSER-SYNC-01` J0 + regression | exact-revision guest daemon `/ui/` on `B01-Desktop-Linux-002` | Vite origin; `:48181` replace; stacked three-column | J0 **pass**; J2/J3/J5/J8 **pass**. Left `:48181` untouched (PID 166715) |

Formal-plan 关闭门: 176px nav、22px titles、44px hits、~1100px 三栏不堆叠 — **true**; 其余诚实墙 secondary — **true**.

Drift negatives: 堆叠三栏 — Dual Track N2 + guest columns `176px 756px 348px`. 回退 T01 min-width — token still `1100px`. Twitter P0 — Dual Track N1 + guest twitter=false. 改 AXIOMS.md — not rewritten. Vite as product origin — walk used daemon `/ui/` only. Canvas v9 not overwritten.

## 2. Validation summary

| Environment | Result |
|---|---|
| Local Node Dual Track | Focused T06 **3/3**. Full `clients/pc/web` **562/562** (80 files) at `6214aa3e`. Development evidence only. |
| `DEV-WIN-GNU-01` | Rust link **not-run** (routed). No Rust files in T06. |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | Implementation HEAD `6214aa3e` required CI [34669451814](https://github.com/agentkernel/cognitive-os/actions/runs/34669451814) **SUCCESS**. This closure commit re-runs required CI before ready/merge. |
| `DEV-LINUX-NATIVE-01` | Exact-revision UI dist `index-uL34SeNM.js` at `6214aa3e`. No kernel-server rebuild (UI-only). |
| `B01-Desktop-Linux-002` | Daemon `:48681` PID **2740931** pin `6214aa3e`. Left `:48181` untouched (PID **166715**). `cognitive dsh web` `:3080` **not-run**. |
| `DEV-WINDOWS-NATIVE-OPC-01` chrome | **not-run** (walk used host Chrome 140 against forwarded guest `/ui/`) |

## 3. Non-claims

Not Gate / release / Profile / B01. No Vite origin. No secret in Git/DOM/report. No numbered migration. Canvas v9 not overwritten. AXIOMS.md not rewritten. T01 `--cp-shell-min-width` not reverted. SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334) not merged. No Twitter/X P0. No fake Activate.

## 4. Unique next

Ready/merge PR [#341](https://github.com/agentkernel/cognitive-os/pull/341) after this closure is on the same required-CI HEAD. Unique next after T06 close: **no remaining ready `P*-T*` implementation task** (Phase 15 Remaining = 0). Do not auto-claim P6. `P7-T07` stays blocked. Evaluation routing OFF.

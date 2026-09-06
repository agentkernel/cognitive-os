# P15-T02 Write Project / ①–⑤ Owner copy vs v9 — closure

- Task: `P15-T02` **done** / slices `P15-T02/D01` **done** + `P15-T02/D02` **done**
- Change class: `implementation-only` (Owner-facing Write/①–⑤ copy vs frozen v9). Persist-before-dispatch Write stays. No `core/specs`. No numbered migration. Canvas v9 file not overwritten. AXIOMS.md not rewritten.
- Lease: `lease/personal/P15-T02/write-owner-copy` → PARALLEL-LANES §3.1 (closed in this delivery)
- Branch / PR: `personal/P15-T02-write-owner-copy` → Draft PR [#337](https://github.com/agentkernel/cognitive-os/pull/337) (ready/merge in this close)
- Dual Track / guest pin: `3528ba28df48f2a949b4189ca295736f607de96e`
- Guest `/ui/` pin: `3528ba28` / SPA `index-V6QE_Yae.js` + `index-C9SPnPVx.css`
- Running report: [P15-T02 report](2026-09-06-personal-p15-t02-report.md)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI / guest `/ui/` close "Write/①–⑤ Owner copy toward v9" only). Not Gate / release / Profile / B01. Windows native chrome JOURNEY **not-run**.
- Evaluation routing: **OFF**. P14-T05/T06 remain **done**. Do not merge leftover SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334). Do not implement T04–T06 from this close.

## 1. Acceptance mapping (formal plan P15-T02 card + D01/D02)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| Write Project / ①–⑤ uses v9 Owner language (过程/成员/测试/联调), not Charter/Process jargon walls | `CreateWizardPage` STEPS titles + PageHeader + headings + labels + `进入 ②`; authority buttons stay `Request preview` / `Write Project` | Dual Track N2 jargon wall (`① Charter`); Vite lecture; developer Dual Track wall as primary chrome | Dual Track N2 **fail** then **pass**; guest J1 ① `创建项目 · ① 项目初始化` / `① 逐项确认这件事` / `标题` / `这件事` / `进入 ②`; ②–⑤ Owner titles |
| No authority → no Write Project; persist-before-dispatch | Preview then confirm on ⑤; Write disabled until preview | Fake Activate; unauthenticated wizard | Dual Track N1 SessionGate; guest J0 unauth fail-closed; J1 Write disabled until preview |
| Fake Activate refused; no Twitter/X P0 | Honesty secondary without `Activate`; no Twitter chrome | Dual Track N3 `\bActivate\b`; `twitter=` true | Dual Track N3 **fail** then **pass**; guest J10 `twitter=false`; 0 Activate |
| `JOURNEY-BROWSER-SYNC-01` J1 + regression J0/J2/J10 | exact-revision guest daemon `/ui/` on `B01-Desktop-Linux-002` | Vite origin; `:48181` replace | J1 **pass**; J0/J2/J10 **pass**. Left `:48181` untouched (PID 166715) |

Formal-plan 关闭门: ①–⑤ 主 chrome 是 v9 Owner 步骤语言 — **true**; Write 仍走 persist-before-dispatch — **true**; 0 假 Activate — **true**.

Drift negatives: Charter/Process jargon wall — Dual Track + guest use 项目初始化 / 流程初始化 / 创建岗位 / 测这一环 / 联合调试. Fake Create/Activate — 0 Activate. Vite as product origin — walk used daemon `/ui/` only. Secret in DOM — one-shot bootstrap file deleted; not in Git/report. Canvas v9 not overwritten. AXIOMS.md not rewritten. P14-T02/T03 functional cards not reopened.

## 2. Validation summary

| Environment | Result |
|---|---|
| Local Node Dual Track | `clients/pc/web` vitest **546/546** (76 files) at worktree ancestor of `3528ba28`. Development evidence only. |
| `DEV-WIN-GNU-01` | Rust link **not-run** (routed). No Rust files in T02. |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | Merge-HEAD required CI is this close after push. Dual Track pin `3528ba28` ubuntu **pass** on ancestor runs. |
| `DEV-LINUX-NATIVE-01` | Exact-revision UI dist `index-V6QE_Yae.js` at `3528ba28`. No kernel-server rebuild (UI-only). |
| `B01-Desktop-Linux-002` | Daemon `:48681` PID **2740931** pin `3528ba28`. Left `:48181` untouched (PID **166715**). `cognitive dsh web` `:3080` **not-run**. |
| `DEV-WINDOWS-NATIVE-OPC-01` chrome | **not-run** (walk used Cursor browser against forwarded guest `/ui/`) |

## 3. Non-claims

Not T03 Projects list/detail vs v9. Not T04–T06. Not Gate / release / Profile / B01. No Vite origin. No secret in Git/DOM/report. No numbered migration. Canvas v9 not overwritten. AXIOMS.md not rewritten. P14-T05/T06 stay **done**. SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334) not merged. SessionGate heading `Create Project` (router not in this lease) is not the authenticated wizard chrome.

## 4. Unique next

Ready/merge PR [#337](https://github.com/agentkernel/cognitive-os/pull/337) after required CI on this closure HEAD (guest pin `3528ba28` already walked). Unique next after T02 close: **claim `P15-T03`** (Projects list/detail vs v9; Dual Track + guest `/ui/` J3 after a **pushed** T03 SHA). Do not claim T04–T06 from this close. Do not auto-claim P6. `P7-T07` stays blocked. Evaluation routing OFF.

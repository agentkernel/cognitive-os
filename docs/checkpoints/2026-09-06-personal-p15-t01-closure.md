# P15-T01 v9 design authority on daemon `/ui/` — closure

- Task: `P15-T01` **done** / slices `P15-T01/D01` **done** + `P15-T01/D02` **done**
- Change class: `product-semantic` (docs reframe: origin = daemon `/ui/`; v9 = design authority/target) + `implementation-only` (shell/Today Owner chrome + `--cp-*` type scale / no-stack). No `core/specs`. No numbered migration. Canvas v9 file not shipped / not overwritten. AXIOMS.md not rewritten.
- Lease: `lease/personal/P15-T01/v9-design-authority-shell` → PARALLEL-LANES §3.1 (closed in this delivery)
- Folded lease: `lease/personal/DOC-P15-V9-BACKLOG/plan-registration` → §3.1 (consumed; T02–T06 remain registered **not-started**)
- Branch / PR: `personal/P15-T01-v9-target` → Draft PR [#335](https://github.com/agentkernel/cognitive-os/pull/335) (ready/merge in this close). Do not merge leftover SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334). Draft [#336](https://github.com/agentkernel/cognitive-os/pull/336) closed as superseded after fold.
- Implementation pin: Dual Track `6b524c53`; guest / required CI `4373d158` (tsc exclude Dual Track tests so `pnpm build` emits `/ui/` dist)
- Guest `/ui/` pin: `4373d158` / SPA `index-D06uNekN.js` + `index-C9SPnPVx.css`
- Running report: [P15-T01 report](2026-09-06-personal-p15-t01-report.md)
- Claim ceiling: `hypothesis` (A7: Dual Track / ordinary CI / guest `/ui/` close "shell/Today Owner chrome toward v9" only). Not Gate / release / Profile / B01. Windows native chrome JOURNEY **not-run**.
- Evaluation routing: **OFF**. P14-T05/T06 remain **done**. Do not claim T03–T06 from this close.

## 1. Acceptance mapping (formal plan P15-T01 card + D01/D02)

| Acceptance item | Implementation | Focused negative(s) | Evidence |
|---|---|---|---|
| Product origin = daemon `/ui/`; v9 = design authority/target; docs no longer say “v9 is not the product” | D01 docs reframe on product/architecture/plan; canvas file stays read-only | Vite-as-product; shipping `.canvas.tsx`; rewriting AXIOMS.md | Dual Track N3/N4; guest walk used `/ui/` only; AXIOMS.md untouched |
| Owner Today chrome is v9 language (今日 / 创建项目 / continue-create), not a developer honesty wall | `TodayPage` + shell tokens `--cp-shell-min-width`; honesty demoted to `details[data-honesty=secondary]` | primary `#main .cp-honesty`; Vite lecture; “not an authority writer” brand | Dual Track N2 **fail** then **pass**; guest J2 h2 `今日`, lede `看清并处理要你拍板的事。`, no primary honesty. Live-packet Today (T06) keeps `创建项目` on empty-home Dual Track, not as a fake packet CTA |
| No stacked horizontal chrome | `--cp-shell-min-width: 1100px`; body 14px / headline 15px | stacked three-column; Twitter/X P0 hero | Dual Track tokens `readFileSync`; guest computed `min-width: 1100px`, body `14px` |
| Fake Activate refused; unauthenticated fail-closed; no Twitter P0 | Session gate + shell | unauth Today; Activate buttons; `twitter=` true | Dual Track N1; guest J0 empty Issue → `management HTTP 401; task HTTP 401`; 0 Activate; J10 `twitter=false` |
| `JOURNEY-BROWSER-SYNC-01` J0/J2/J10 + Phase 14 regression | exact-revision guest daemon `/ui/` on `B01-Desktop-Linux-002` | Vite origin; `:48181` replace | J0/J2/J10 **pass** (re-walked 11:14 after stall); J1 chrome / J18 / J19 / Knowledge / Settings L1 regression **pass**. Left `:48181` untouched (PID 166715) |

Formal-plan 关闭门: Owner 在 Today 看到 v9 语言 — **true** (live T06 packets remain packets; empty-home Dual Track still `创建项目`); 无堆叠横滚 — **true**; 假 Activate 仍拒 — **true**; 未认证 fail-closed — **true**; 无 Twitter P0 — **true**; 文档不再写「v9 不是产品」 — **true**.

Drift negatives: canvas/Vite as product origin — walk used daemon `/ui/` only. Mock complete-job — not copied. Fake Activate — Dual Track + guest 0 buttons. Weaken A1–A5 — unchanged. Reopen Phase 14 — T05/T06 stay **done**. 19 modules in one task — T02–T06 remain not-started. Windows chrome **not-run**.

## 2. Validation summary

| Environment | Result |
|---|---|
| Local Node Dual Track | `clients/pc/web` vitest **543/543** (75 files) at `6b524c53` / re-run ancestor of `4373d158`. Development evidence only. |
| `DEV-WIN-GNU-01` | Rust link **not-run** (routed). No Rust files in T01. |
| `CI-UBUNTU-01` / `CI-WINDOWS-MSVC-01` | Deployed SHA [34006761447](https://github.com/agentkernel/cognitive-os/actions/runs/34006761447) **SUCCESS** at `4373d158`. Merge-HEAD required CI is this close after push. |
| `DEV-LINUX-NATIVE-01` | Exact-revision `kernel-server` ELF + UI dist `index-D06uNekN.js` at `4373d158`. |
| `B01-Desktop-Linux-002` | Daemon `:48681` PID **2740931** pin `4373d158`. Left `:48181` untouched (PID **166715**). `cognitive dsh web` `:3080` **not-run** (stale Path B after daemon replace). |
| `DEV-WINDOWS-NATIVE-OPC-01` chrome | **not-run** (walk used Cursor browser against forwarded guest `/ui/`) |

## 3. Non-claims

Not T02 Write/①–⑤ Owner copy. Not T03–T06. Not Gate / release / Profile / B01. No Vite origin. No secret in Git/DOM/report. No numbered migration. Canvas v9 not overwritten. AXIOMS.md not rewritten. P14-T05/T06 stay **done**. SNAP [#334](https://github.com/agentkernel/cognitive-os/pull/334) not merged.

## 4. Unique next

Ready/merge PR [#335](https://github.com/agentkernel/cognitive-os/pull/335) after required CI on this closure HEAD (guest pin `4373d158` already green). Unique next after T01 close: **claim `P15-T02`** (Write Project / Dual Track ①–⑤ Owner copy vs v9; guest J1 after a **pushed** T02 SHA). Do not claim T03–T06 from this close. Do not auto-claim P6. `P7-T07` stays blocked. Evaluation routing OFF.

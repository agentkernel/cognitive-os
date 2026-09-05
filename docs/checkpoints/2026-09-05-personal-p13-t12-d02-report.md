# P13-T12/D02 visual / a11y / rendered `/ui/` qualification — running report

Incremental log per `TEST-REPORT-INCREMENTAL-01`. `not-run` is never pass. Claim ceiling `hypothesis`. A7: host dump-dom / CDP against Linux guest `/ui/` is implementation evidence only — not Windows native chrome, not Gate, release, or Profile.

- Task: `P13-T12` / slice `P13-T12/D02`
- Branch: `personal/P13-T12-D02-visual-qualification` (worktree `D:\\agent-kernel-wt-P13-T12-D02` from `origin/main@c8691923`)
- Lease: `lease/personal/P13-T12/visual-qualification`
- Change class: `implementation-only` documentation (judgement sheet + report; no product CSS/IA/canvas edit; no T11 product/UI code)
- Product origin: daemon-served `/ui/` on pushed exact revision `c8691923cd3988f0ffee9123752e073480aea5e9`. Vite preview is not the product. Canvas screenshots are never acceptance.
- Sibling isolation: did not use `D:\\agent-kernel-wt-P13-T11` or `personal/P13-T11-reflection`. Did not patch `app.css` ≤1279 px stack (spec §13-a).
- Unique next on main for T11 (close Draft PR #320) is preserved; this lease does not claim T11.

## Pin

| Field | Value |
|---|---|
| Exact `/ui/` revision (pushed SHA) | `c8691923cd3988f0ffee9123752e073480aea5e9` (`origin/main` at claim; Linux worktree `~/cognitiveos-personal-worktrees/p13-t12-c8691923`) |
| Guest daemon environment | `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2`) bind `127.0.0.1:48786` (did not touch `:48181` / `:48681` / `:39245`) |
| Host browser | Chrome **151.0.7922.174** on `DEV-WIN-GNU-01` via SSH tunnel `48786`; headless CDP dump-dom |
| NVDA | **not installed** on this host (`C:\\Program Files\\NVDA` absent) — Grid E = `not-run` |
| Windows native chrome | `DEV-WINDOWS-NATIVE-OPC-01` not provisioned — skipped this session per owner |
| D02 running report | this file |
| Off-Git cell log digest | `sha256:c7b24416b6210df99b65a131d0b71a8fe5f3058a9baa3a443ae1b3c202e28188` (`d:/tmp/p13-t12-review/out/cells.json`; 248 harness rows including 5 `module-route` probes; not committed) |

## Units

| Unit | Result | Env | Revision | Notes |
|---|---|---|---|---|
| Worktree + lease claim | **pass** | `DEV-WIN-GNU-01` | `c8691923` | A8: main leftover skills not mixed. Sibling T11 worktree not used. |
| Host Chrome present | **pass** | `DEV-WIN-GNU-01` | — | 151.0.7922.174 |
| NVDA present | **not-run** | `DEV-WIN-GNU-01` | — | no NVDA install path |
| Windows native OPC chrome | **not-run** | `DEV-WINDOWS-NATIVE-OPC-01` | — | skipped this session |
| SSH `DEV-LINUX-NATIVE-01` | **pass** | `DEV-LINUX-NATIVE-01` | — | Existing daemons `:48181` / `:48681` / `:39245` left untouched |
| Exact-revision guest `/ui/` | **pass** | `DEV-LINUX-NATIVE-01` | `c8691923` | First copy to `data/ui` returned HTTP 503 (`data_dir` is `data/cognitiveos/ui`); recopy then `GET /ui/` HTTP 200, 655 bytes. Runtime `~/cos-wt/p13-t12-rt`. |
| Session gate via CDP | **pass** | host Chrome → tunnel | `c8691923` | Bootstrap file scp to `%TEMP%` (71 bytes); never printed. Form submit un-gated to `[data-page=opc-today]`. |
| Rendered Chrome / dump-dom grids | **pass** (executed) | `DEV-LINUX-NATIVE-01` + host Chrome | `c8691923` | Executable cells filled; remainder honest `not-run`. |

## Counters (checklist §9; after superseding rows)

| Grid | Cells | pass | fail | partial | not-run |
|---|---:|---:|---:|---:|---:|
| §1 modules | 19 | 7 | 1 | 1 | 10 |
| §2 State Lab 9×9 | 81 | 0 | 81 | 0 | 0 |
| §3 keyboard / focus | 57 | 7 | 5 | 10 | 35 |
| §4 200% / narrow | 36 | 5 | 15 | 0 | 16 |
| §5 themes | 40 | 18 | 0 | 0 | 22 |
| §6 NVDA | 10 | 0 | 0 | 0 | 10 |

Harness also recorded 5 `module-route` probes (all `pass`) that are not checklist rows.

## Material observations (do not patch in this lease)

1. **State Lab (M-STATE + Grid A 81× fail).** Settings → Advanced mounts 81 `[data-state-lab-cell]` shared widgets, not real surface layouts (spec §9.2). Empty cells have 0 primaries; several `unknown` cells show `0`.
2. **Grid C V2/V3/V4 fail** on the five executable surfaces: `app.css` ≤1279 px stacks columns (spec §13-a). V1 1440@100% `pass` (`areas="strip strip strip" "side main rail"`, clipped=0). No CSS patch.
3. **Grid B K1 fail** (supersede): first heading is brand `h1` "CognitiveOS Personal"; space title is `h2` (Today / Projects / …). Spec §13-m.
4. **No live Project** on the disposable runtime → members/runs/outputs/hitl routes and later wizard steps `not-run`. Empty Home / Projects / Knowledge / Settings / Create ① were judged on what rendered.
5. **Themes L/D/HC pass** on executable surfaces (0 on-screen text pairs under 4.5:1). FC / Windows High Contrast `not-run` until P13-T13.
6. **NVDA 10× `not-run`** — NVDA not installed; no invented environment ID.
7. **No fake Activate / Publish / Team / Inbox / X** on dumped L1 surfaces. Empty Home has one `Start create` primary link.

## Superseding rows

Harness first-pass judgements for M-CREATE-2..5 (`partial`), M-TODAY / M-TODAY-INCOMPLETE (`pass`), M-LIVE-PROJECT (`partial`), and Grid B K1 (`pass`) were superseded from dump-dom (this table). The checklist carries the superseded values.

## Cells

| Grid | Id | Judgement | Env | Rev | Reason |
|---|---|---|---|---|---|
| `module` | `M-X` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | no X chrome in L1/Today/Settings/rail |
| `module` | `M-SETTINGS` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | settings page present; advanced collapsed by default until opened |
| `module` | `M-EMPTY` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | Today rendered; empty/honesty judged on live daemon list |
| `module` | `M-SHELL` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | L1 Today/Projects/Knowledge; skip link; #main |
| `module` | `M-PROJECTS` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | projects page host |
| `module` | `M-KNOWLEDGE` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | knowledge page host |
| `module` | `M-CREATE-1` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | create wizard host |
| `module` | `M-CREATE-2` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | supersede harness partial: wizard remained create-init; step ② not driven |
| `module` | `M-CREATE-3` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | supersede harness partial: wizard remained create-init; step ③ not driven |
| `module` | `M-CREATE-4` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | supersede harness partial: wizard remained create-init; step ④ not driven |
| `module` | `M-CREATE-5` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | supersede harness partial: wizard remained create-init; step ⑤ not driven |
| `module` | `M-TODAY-INCOMPLETE` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | supersede harness pass: no creating Project row; empty Home is M-EMPTY |
| `module` | `M-TODAY` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | supersede harness pass: no live Project; packets/overview not instantiated |
| `module` | `M-LIVE-PROJECT` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | supersede harness partial: no live Project id |
| `module` | `M-ADD-MEMBER` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id in disposable runtime — route #/projects/:id/members/new not instantiated |
| `module` | `M-MEMBER-CONFIG` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id in disposable runtime |
| `module` | `M-HITL` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no pending preview id in disposable runtime |
| `module` | `M-CHAT-CANVAS` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | wizard hides rail (spec §13-i vs v9); judged on present chrome |
| `module` | `M-STATE` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab grid cells=81; hidden-by-default advanced; cells are shared widgets not real surfaces (spec §9.2) |
| `grid-a` | `today:loading` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `today:empty` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); empty primary count 0 (need exactly 1) |
| `grid-a` | `today:working` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `today:error` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `today:success` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `today:partial` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `today:blocked` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `today:unknown` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); unknown shows 0 |
| `grid-a` | `today:offline` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `create:loading` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `create:empty` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); empty primary count 0 (need exactly 1) |
| `grid-a` | `create:working` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `create:error` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `create:success` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `create:partial` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `create:blocked` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `create:unknown` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); unknown shows 0 |
| `grid-a` | `create:offline` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `projects:loading` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `projects:empty` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); empty primary count 0 (need exactly 1) |
| `grid-a` | `projects:working` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `projects:error` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `projects:success` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `projects:partial` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `projects:blocked` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `projects:unknown` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); unknown shows 0 |
| `grid-a` | `projects:offline` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `members:loading` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `members:empty` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); empty primary count 0 (need exactly 1) |
| `grid-a` | `members:working` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `members:error` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `members:success` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `members:partial` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `members:blocked` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `members:unknown` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); unknown shows 0 |
| `grid-a` | `members:offline` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `runs:loading` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `runs:empty` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); empty primary count 0 (need exactly 1) |
| `grid-a` | `runs:working` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `runs:error` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `runs:success` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `runs:partial` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `runs:blocked` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `runs:unknown` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); unknown shows 0 |
| `grid-a` | `runs:offline` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `outputs:loading` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `outputs:empty` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); empty primary count 0 (need exactly 1) |
| `grid-a` | `outputs:working` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `outputs:error` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `outputs:success` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `outputs:partial` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `outputs:blocked` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `outputs:unknown` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); unknown shows 0 |
| `grid-a` | `outputs:offline` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `hitl:loading` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `hitl:empty` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); empty primary count 0 (need exactly 1) |
| `grid-a` | `hitl:working` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `hitl:error` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `hitl:success` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `hitl:partial` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `hitl:blocked` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `hitl:unknown` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); unknown shows 0 |
| `grid-a` | `hitl:offline` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `knowledge:loading` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `knowledge:empty` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); empty primary count 0 (need exactly 1) |
| `grid-a` | `knowledge:working` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `knowledge:error` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `knowledge:success` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `knowledge:partial` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `knowledge:blocked` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `knowledge:unknown` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); unknown shows 0 |
| `grid-a` | `knowledge:offline` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `settings:loading` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `settings:empty` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); empty primary count 0 (need exactly 1) |
| `grid-a` | `settings:working` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `settings:error` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `settings:success` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `settings:partial` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `settings:blocked` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-a` | `settings:unknown` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2); unknown shows 0 |
| `grid-a` | `settings:offline` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | state-lab cell is a shared state widget, not the real surface layout (spec §9.2) |
| `grid-b` | `today:K1` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | supersede harness pass: first heading is brand h1 CognitiveOS Personal; space title is h2 |
| `grid-b` | `today:K2` | `partial` | DEV-LINUX-NATIVE-01 | `c8691923` | Tab order observed via skip-link and nav; full 40-stop walk is pointer-emulated not OS-Tab |
| `grid-b` | `today:K3` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | focus outline visible on a chrome control |
| `grid-b` | `today:K4` | `partial` | DEV-LINUX-NATIVE-01 | `c8691923` | widget keys (tablist/listbox/dialog) not fully driven without live widgets on this runtime |
| `grid-b` | `today:K5` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no invalid-field error injected on this disposable runtime |
| `grid-b` | `today:K6` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no list filter/selection persistence scenario on disposable empty runtime |
| `grid-b` | `create:K1` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | supersede harness pass: first heading is brand h1 CognitiveOS Personal; space title is h2 |
| `grid-b` | `create:K2` | `partial` | DEV-LINUX-NATIVE-01 | `c8691923` | Tab order observed via skip-link and nav; full 40-stop walk is pointer-emulated not OS-Tab |
| `grid-b` | `create:K3` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | focus outline visible on a chrome control |
| `grid-b` | `create:K4` | `partial` | DEV-LINUX-NATIVE-01 | `c8691923` | widget keys (tablist/listbox/dialog) not fully driven without live widgets on this runtime |
| `grid-b` | `create:K5` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no invalid-field error injected on this disposable runtime |
| `grid-b` | `create:K6` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no list filter/selection persistence scenario on disposable empty runtime |
| `grid-b` | `projects:K1` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | supersede harness pass: first heading is brand h1 CognitiveOS Personal; space title is h2 |
| `grid-b` | `projects:K2` | `partial` | DEV-LINUX-NATIVE-01 | `c8691923` | Tab order observed via skip-link and nav; full 40-stop walk is pointer-emulated not OS-Tab |
| `grid-b` | `projects:K3` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | focus outline visible on a chrome control |
| `grid-b` | `projects:K4` | `partial` | DEV-LINUX-NATIVE-01 | `c8691923` | widget keys (tablist/listbox/dialog) not fully driven without live widgets on this runtime |
| `grid-b` | `projects:K5` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no invalid-field error injected on this disposable runtime |
| `grid-b` | `projects:K6` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no list filter/selection persistence scenario on disposable empty runtime |
| `grid-b` | `knowledge:K1` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | supersede harness pass: first heading is brand h1 CognitiveOS Personal; space title is h2 |
| `grid-b` | `knowledge:K2` | `partial` | DEV-LINUX-NATIVE-01 | `c8691923` | Tab order observed via skip-link and nav; full 40-stop walk is pointer-emulated not OS-Tab |
| `grid-b` | `knowledge:K3` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | focus outline visible on a chrome control |
| `grid-b` | `knowledge:K4` | `partial` | DEV-LINUX-NATIVE-01 | `c8691923` | widget keys (tablist/listbox/dialog) not fully driven without live widgets on this runtime |
| `grid-b` | `knowledge:K5` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no invalid-field error injected on this disposable runtime |
| `grid-b` | `knowledge:K6` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no list filter/selection persistence scenario on disposable empty runtime |
| `grid-b` | `settings:K1` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | supersede harness pass: first heading is brand h1 CognitiveOS Personal; space title is h2 |
| `grid-b` | `settings:K2` | `partial` | DEV-LINUX-NATIVE-01 | `c8691923` | Tab order observed via skip-link and nav; full 40-stop walk is pointer-emulated not OS-Tab |
| `grid-b` | `settings:K3` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | focus outline visible on a chrome control |
| `grid-b` | `settings:K4` | `partial` | DEV-LINUX-NATIVE-01 | `c8691923` | widget keys (tablist/listbox/dialog) not fully driven without live widgets on this runtime |
| `grid-b` | `settings:K5` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no invalid-field error injected on this disposable runtime |
| `grid-b` | `settings:K6` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no list filter/selection persistence scenario on disposable empty runtime |
| `grid-b` | `members:K1` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `members:K2` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `members:K3` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `members:K4` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `members:K5` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `members:K6` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `runs:K1` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `runs:K2` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `runs:K3` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `runs:K4` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `runs:K5` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `runs:K6` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `outputs:K1` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `outputs:K2` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `outputs:K3` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `outputs:K4` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `outputs:K5` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `outputs:K6` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `hitl:K1` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `hitl:K2` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `hitl:K3` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `hitl:K4` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `hitl:K5` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `hitl:K6` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project / preview id on disposable runtime |
| `grid-b` | `shell:S1` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | skip link present |
| `grid-b` | `shell:S2` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | ⌘K opened a palette/dialog |
| `grid-b` | `shell:S3` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | rail composer hidden on empty Home; no live Project rail |
| `grid-c` | `today:V1` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | stacked=false clipped=0 overflow=0 areas="strip strip strip" "side main rail" |
| `grid-c` | `today:V2` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=720 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `today:V3` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=1100 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `today:V4` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=960 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `create:V1` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | stacked=false clipped=0 overflow=0 areas="strip strip strip" "side main rail" |
| `grid-c` | `create:V2` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=720 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `create:V3` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=1100 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `create:V4` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=960 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `projects:V1` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | stacked=false clipped=0 overflow=0 areas="strip strip strip" "side main rail" |
| `grid-c` | `projects:V2` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=720 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `projects:V3` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=1100 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `projects:V4` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=960 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `knowledge:V1` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | stacked=false clipped=0 overflow=0 areas="strip strip strip" "side main rail" |
| `grid-c` | `knowledge:V2` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=720 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `knowledge:V3` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=1100 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `knowledge:V4` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=960 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `settings:V1` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | stacked=false clipped=0 overflow=0 areas="strip strip strip" "side main rail" |
| `grid-c` | `settings:V2` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=720 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `settings:V3` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=1100 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `settings:V4` | `fail` | DEV-LINUX-NATIVE-01 | `c8691923` | columns stacked at cssW=960 (spec §6 / §13-a app.css ≤1279); pageOverflow=0 |
| `grid-c` | `members:V1` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `members:V2` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `members:V3` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `members:V4` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `runs:V1` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `runs:V2` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `runs:V3` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `runs:V4` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `outputs:V1` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `outputs:V2` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `outputs:V3` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `outputs:V4` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `hitl:V1` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `hitl:V2` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `hitl:V3` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-c` | `hitl:V4` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-d` | `today:L` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (L) |
| `grid-d` | `today:D` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (D) |
| `grid-d` | `today:HC` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (HC) |
| `grid-d` | `today:FC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | forced-colors / Windows High Contrast not forced; Windows native chrome = P13-T13 / DEV-WINDOWS-NATIVE-OPC-01 |
| `grid-d` | `create:L` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (L) |
| `grid-d` | `create:D` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (D) |
| `grid-d` | `create:HC` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (HC) |
| `grid-d` | `create:FC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | forced-colors / Windows High Contrast not forced; Windows native chrome = P13-T13 / DEV-WINDOWS-NATIVE-OPC-01 |
| `grid-d` | `projects:L` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (L) |
| `grid-d` | `projects:D` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (D) |
| `grid-d` | `projects:HC` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (HC) |
| `grid-d` | `projects:FC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | forced-colors / Windows High Contrast not forced; Windows native chrome = P13-T13 / DEV-WINDOWS-NATIVE-OPC-01 |
| `grid-d` | `knowledge:L` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (L) |
| `grid-d` | `knowledge:D` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (D) |
| `grid-d` | `knowledge:HC` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (HC) |
| `grid-d` | `knowledge:FC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | forced-colors / Windows High Contrast not forced; Windows native chrome = P13-T13 / DEV-WINDOWS-NATIVE-OPC-01 |
| `grid-d` | `settings:L` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (L) |
| `grid-d` | `settings:D` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (D) |
| `grid-d` | `settings:HC` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (HC) |
| `grid-d` | `settings:FC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | forced-colors / Windows High Contrast not forced; Windows native chrome = P13-T13 / DEV-WINDOWS-NATIVE-OPC-01 |
| `grid-d` | `shell:L` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (L) |
| `grid-d` | `shell:D` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (D) |
| `grid-d` | `shell:HC` | `pass` | DEV-LINUX-NATIVE-01 | `c8691923` | 0 on-screen text pairs under 4.5:1 (HC) |
| `grid-d` | `shell:FC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | forced-colors / Windows High Contrast not forced; Windows native chrome = P13-T13 / DEV-WINDOWS-NATIVE-OPC-01 |
| `grid-d` | `members:L` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-d` | `members:D` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-d` | `members:HC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-d` | `members:FC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | Windows High Contrast not forced (P13-T13) |
| `grid-d` | `runs:L` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-d` | `runs:D` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-d` | `runs:HC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-d` | `runs:FC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | Windows High Contrast not forced (P13-T13) |
| `grid-d` | `outputs:L` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-d` | `outputs:D` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-d` | `outputs:HC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-d` | `outputs:FC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | Windows High Contrast not forced (P13-T13) |
| `grid-d` | `hitl:L` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-d` | `hitl:D` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-d` | `hitl:HC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | no live Project id on disposable runtime |
| `grid-d` | `hitl:FC` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | Windows High Contrast not forced (P13-T13) |
| `grid-e` | `N1` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | NVDA not installed on DEV-WIN-GNU-01 (no C:\Program Files\NVDA); do not invent an environment ID |
| `grid-e` | `N2` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | NVDA not installed on DEV-WIN-GNU-01 (no C:\Program Files\NVDA); do not invent an environment ID |
| `grid-e` | `N3` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | NVDA not installed on DEV-WIN-GNU-01 (no C:\Program Files\NVDA); do not invent an environment ID |
| `grid-e` | `N4` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | NVDA not installed on DEV-WIN-GNU-01 (no C:\Program Files\NVDA); do not invent an environment ID |
| `grid-e` | `N5` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | NVDA not installed on DEV-WIN-GNU-01 (no C:\Program Files\NVDA); do not invent an environment ID |
| `grid-e` | `N6` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | NVDA not installed on DEV-WIN-GNU-01 (no C:\Program Files\NVDA); do not invent an environment ID |
| `grid-e` | `N7` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | NVDA not installed on DEV-WIN-GNU-01 (no C:\Program Files\NVDA); do not invent an environment ID |
| `grid-e` | `N8` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | NVDA not installed on DEV-WIN-GNU-01 (no C:\Program Files\NVDA); do not invent an environment ID |
| `grid-e` | `N9` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | NVDA not installed on DEV-WIN-GNU-01 (no C:\Program Files\NVDA); do not invent an environment ID |
| `grid-e` | `N10` | `not-run` | DEV-LINUX-NATIVE-01 | `c8691923` | NVDA not installed on DEV-WIN-GNU-01 (no C:\Program Files\NVDA); do not invent an environment ID |

## Non-claims

- Not Gate, release, Profile, B01, Agent-benefit, or `P11-T15`.
- Not Windows native chrome qualification (`P13-T13`).
- Not a CSS/IA fix for spec §13-a or §13-m.
- Canvas v9 ≠ product. Vite ≠ product origin.
- PERS-PR-052 stays `not-run` until `P13-T13` provisions `DEV-WINDOWS-NATIVE-OPC-01` (statement includes native backfill).

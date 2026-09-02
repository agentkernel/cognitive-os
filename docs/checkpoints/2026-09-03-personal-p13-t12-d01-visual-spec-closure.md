# P13-T12/D01 Visual UI spec + v9 comparison checklist — closure

- Task: `P13-T12` (stays **in-progress**; `P13-T12/D02` is `ready` and not claimed) / slice `P13-T12/D01` **done**
- Branch: `personal/P13-T12-D01-visual-spec` (remote deleted after merge; worktree `D:/agent-kernel-wt-p13-t12` removed)
- Lease: `lease/personal/P13-T12/visual-spec` → PARALLEL-LANES §3.1 (closed 2026-09-03)
- PR: [#308](https://github.com/agentkernel/cognitive-os/pull/308) merged at `main@3680b742`
- Required CI: [33669225379](https://github.com/agentkernel/cognitive-os/actions/runs/33669225379) **SUCCESS** at `bd87b8ee` (resolve 2s, ubuntu 3m35s, windows 17m9s, required-ci 3s)
- Change class: `implementation-only` documentation (unchanged product/IA/machine contracts; no CSS, canvas, code or test change)
- Claim ceiling: `hypothesis`
- Running report: [2026-09-03-personal-p13-t12-d01-visual-spec-report.md](2026-09-03-personal-p13-t12-d01-visual-spec-report.md)

## Delivered

- [`personal/docs/architecture/personal-2.0-opc-visual-ui-spec.md`](../../personal/docs/architecture/personal-2.0-opc-visual-ui-spec.md) — Apple-led Visual UI specification for daemon `/ui/`: typography scale, 4 pt grid and density, light / dark / high-contrast colour roles with contrast targets and the nine-state → seven-category tone mapping, locked three-column shell with the narrow-window horizontal-scroll rule (columns never stack) and 200% behaviour, focus ring and keyboard order / widget patterns / NVDA key paths, motion and reduced motion, nine State Lab states as visual patterns on the nine surfaces (with `data-page` / `data-region` host selectors), component states, a *proposed* token table (not applied), and twelve recorded-not-decided drift items. Everything is expressed against existing `clients/pc/web/src/tokens.css` names; it states explicitly that it is not a second design system.
- [`personal/docs/architecture/personal-2.0-opc-v9-ui-comparison-checklist.md`](../../personal/docs/architecture/personal-2.0-opc-v9-ui-comparison-checklist.md) — all 19 `00-maintenance-index.md` table-A modules (19/19) mapped to v9 scene(s), table-B authority, `/ui/` route + component/selector, daemon fact / honest state, owning P12/P13 card and a judgement column pre-filled `not-run`; plus the D02 cell grids: 81 nine × nine, 57 keyboard/focus, 36 200%/narrow, 40 host-theme (incl. `forced-colors`), 10 NVDA paths; D02 pin block; module-level drift.
- Links from `personal/docs/architecture/README.md`; bilingual `dev.architecture-overview` pointer + sources + fingerprints (`user.system-overview` fingerprint-only); plan / PROGRESS / plan.md / trace synchronised. Dev-prep index and every REFRAME-owned path untouched.

## Non-claims

Every checklist judgement is `not-run` (0 pass / 0 fail) until `P13-T12/D02` records it on one exact daemon `/ui/` revision from a registered host. NVDA / 200% / host-theme / rendered nine × nine remain **not-run**. No IA change, no canvas regeneration, no CSS edit. Not Gate, release, Profile, Windows qualification, `P11-T15`, or Agent-benefit.

## Observed, not decided (for the owner / plan)

- Current `app.css` stacks strip / side / main / rail at ≤ 1279 px; product docs, scope §3.1, design 09 and v9 require horizontal scroll with the three columns locked. No P13 card currently owns that CSS change; D02's narrow / 200% cells will record `fail` until it lands (spec §13-a; checklist Grid C note).
- `/ui/` hides the Assistant rail on the create wizard and creating-only Today; v9 hides it only on empty Home and scope §3.1 says the create ring's chat defaults to the Assistant (spec §13-i).
- Copy locale: product docs / v9 use zh-CN surface and tab terms; `/ui/` renders English. The checklist judges order and meaning, not language (spec §13-h).

## Unique next

Main line unchanged: claim `P13-T02` (hosted DSH real Attempt loop) and/or `P13-T03` (hidden Pi real inference) in separate leases/worktrees. `P13-T12/D02` waits for `P13-T04/D02`, `P13-T05/D02`, `P13-T07/D01`, `P13-T08/D02` and needs a task-scoped continuation lease. Do not claim `P11-T15`.

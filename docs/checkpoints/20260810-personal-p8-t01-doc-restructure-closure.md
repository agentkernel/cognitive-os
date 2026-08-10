# P8-T01 documentation restructure — closure checkpoint (draft)

- Task: `P8-T01`
- Branch: `personal/P8-T01-doc-restructure`
- Lease: `lease/personal/P8-T01/doc-restructure`
- Change class: `product-semantic + corrective + structural` documentation-only
- Date: 2026-08-10

## Acceptance mapping

| Acceptance item (formal plan) | Evidence |
|---|---|
| Single axioms document + Operating Model §8 deferral | `docs/governance/AXIOMS.md`; DOM §8; `AGENTS.md` thin restatement; ADR-0041 |
| AGENTS/.cursor thin entry; no divergent invariant lists | `AGENTS.md`; `.cursor/rules/00-*.mdc`; PROJECT-IDENTITY + project-scope.yaml |
| PARALLEL-LANES closed history externalized | `docs/plan/PARALLEL-LANES-CLOSED.md`; §3.1 summary pointer |
| ADR-0008 superseded label; milestone prompts non-executable; docs-sync status label | ADR-0008 header; `docs/prompts/milestone-m*.md` banners; docs-sync-contract |
| PROGRESS Layer 3 / TEST-ENVIRONMENTS / plan structural repair + Phase 8/9 | PERSONAL-DEVELOPMENT-PLAN; PROGRESS; PERSONAL-TEST-ENVIRONMENTS |
| Whitepaper OS / gap / pillars / ecosystem alignment | `CognitiveOS-Architecture.md` Personal 对齐章; root review/RFC currency notes |
| Product/architecture extensions + headroom + personal-2.0-scope | product + architecture personal docs listed in lease |
| ADR-0041..0045 | `docs/adr/0041`–`0045` |
| plan.md + personal-trace sync | `plan.md` Phase 8/9 cards; PERS-PR-031..034 |
| Local validation | `pnpm run check:consistency` OK; `git diff --check` OK |
| Required Ubuntu/Windows CI | **pending** on Draft PR (not-run until push) |
| Gate/release/Profile | **non-claim** |

## Non-claims

No specs/conformance edits. No implementation. No Gate, release, or Profile claim.

## Next action

Push branch, open/update Draft PR, record required CI, then complete ready/merge only after CI green and acceptance remains complete.

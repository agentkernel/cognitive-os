---
doc_id: ai.docs-impact
locale: en
kind: reference
audience: [ai]
status: implemented
generated: false
sources:
  - path: personal/handbook/_meta/source-map.json
  - path: docs/standards/docs-sync-contract.md
  - path: tools/src/docs-sync-gate.mjs
    symbols: ["routeChangedPaths", "decideDocsSync"]
fingerprint: "sha256:894db3102b8099c13368a4b3659b1c4231eb88fb04def0a7ccad99779703918d"
non_claims:
  - This page adapts the docs-sync contract for the handbook; the contract itself owns legacy-documentation obligations.
---

# Docs impact

Documentation synchronization is an **enforced pre-commit/pre-push/pre-merge
obligation** (contract §2), not a courtesy. The machine gate is
`node tools/src/docs-sync-gate.mjs --staged|--push|--range`: it routes changed
paths through the source map, runs the handbook check set on any hit, and fails
a change set whose mapped sources carry no handbook update. Enable the repo
hooks once per clone: `pnpm run hooks:install`. The only escape for a genuinely
documentation-neutral change is `DOCS_IMPACT_NONE="<concrete reason>"`, and the
same reason must be recorded in the commit/PR description.

Decide documentation impact **before** finishing a change, using
[`personal/handbook/_meta/source-map.json`](../../_meta/source-map.json):

1. Match every changed path against the source-map rules; collect the mapped
   `doc_ids`.
2. For each mapped page: hand-written pages are updated by editing the page and
   refreshing its fingerprint (`node tools/src/fill-handbook-fingerprints.mjs`);
   generated pages are refreshed only by `node tools/src/generate-handbook.mjs`.
3. User-visible behavior (CLI verbs/flags, configuration files, error surfaces,
   installation, recovery, security boundaries) must update the user and reference
   trees; architectural, data, protocol, authority, or test-environment changes must
   update the developer and AI trees.
4. A brand-new tracked file must be classified in
   [`source-coverage.json`](../../_meta/source-coverage.json) or the handbook check
   fails.
5. If a change genuinely affects no documentation, the PR description must record a
   concrete `docs-impact: none — <reason>` line, not silence.
6. Affected docs and code belong to the same formal task and the same PR.

## Personal 2.0 semantic routes

The source map deliberately makes the adopted target impossible to change
silently:

- `personal-2-baseline` routes the canonical Personal 2.0 product and
  architecture baseline;
- `personal-2-desktop-account-hub` routes ADR-0055/0056 plus Account Hub,
  Provider, and Web UI product/architecture sources;
- `personal-2-agent-supervision` routes the bilingual Agent conversation
  design and Agent Shell/adapter/multi-Agent/recovery/learning architecture;
- `personal-2-mcp-family` routes ADR-0037/0057 plus the bilingual MCP-family,
  cognitive-resource, and Resource Manager sources.

For every hit, preserve two explicit columns of truth: **current Linux
1.0/current API** (six-family, Pi-qualified, current `/ui/` at
`clients/pc/web/`) and **adopted Personal 2.0 target** (`Requires-backend` or
`Requires-core`). Never infer implementation of Account Hub import, the target
desktop redesign, MCP seventh-family/federation, vendor conversation adapters,
Goal/Plan, or multi-Agent supervision from design adoption.

The localized canonical design files route to the same bilingual handbook
`doc_id`s. Update both locales. Refresh fingerprints only for hand-written
pages after all mapped sources exist; do not hand-edit or regenerate unrelated
generated pages merely to hide fingerprint drift.

Verification set for any documentation-affecting change:

```powershell
node tools/src/docs-sync-gate.mjs --staged   # or --push / --range
node tools/src/generate-handbook.mjs --check
node tools/src/check-handbook.mjs
pnpm run check:consistency
git diff --check
```

Legacy documentation (`docs/**`) keeps its own obligations under
[`docs-sync-contract.md`](../../../../docs/standards/docs-sync-contract.md); the handbook never absorbs or
replaces them. When a canonical source and a handbook page conflict, fix the handbook
page in the same delivery — never "fix" the canonical source to match documentation.

---
doc_id: ai.docs-impact
locale: en
kind: reference
audience: [ai]
status: implemented
generated: false
sources:
  - path: handbook/_meta/source-map.json
  - path: docs/standards/docs-sync-contract.md
  - path: tools/src/docs-sync-gate.mjs
    symbols: ["routeChangedPaths", "decideDocsSync"]
fingerprint: "sha256:a3c9fdac9e7960bbc331132ef313e186e8a85917666338cad85e47200fd7cefd"
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
[`handbook/_meta/source-map.json`](../../_meta/source-map.json):

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

Verification set for any documentation-affecting change:

```powershell
node tools/src/docs-sync-gate.mjs --staged   # or --push / --range
node tools/src/generate-handbook.mjs --check
node tools/src/check-handbook.mjs
pnpm run check:consistency
git diff --check
```

Legacy documentation (`docs/**`) keeps its own obligations under
[`docs-sync-contract.md`](../../../docs/standards/docs-sync-contract.md); the handbook never absorbs or
replaces them. When a canonical source and a handbook page conflict, fix the handbook
page in the same delivery — never "fix" the canonical source to match documentation.

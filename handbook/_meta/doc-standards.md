---
doc_id: meta.doc-standards
locale: en
kind: meta
audience: [developer, ai]
generated: false
---

# Handbook documentation standards

（本页为 locale-neutral 元规范：以英文为准，关键约定附中文说明。）

## Position in the repository

The handbook is an **informative derived layer**. It never owns axioms, tasks,
Gates, contracts, error codes, state machines, or current status. Canonical
sources win every conflict, and the handbook page must be corrected in the same
delivery（canonical 来源恒优先；冲突在同一交付内修正手册）。

## Structure

- One locale-independent `doc_id` per document; identical content commitments in
  `handbook/en/**` and `handbook/zh-CN/**` (full parallel trees, no mixed-language
  pages). Meta pages under `handbook/_meta/` are locale-neutral single files.
- Every document is registered in [`manifest.json`](./manifest.json); an
  unregistered markdown file under `handbook/` fails the checker (HB004).
- Frontmatter must validate against
  [`handbook-frontmatter.schema.json`](./handbook-frontmatter.schema.json):
  `doc_id`, `locale`, `kind`, `audience`, `generated`, plus — for every
  fact-bearing kind — `status`, `sources`, `fingerprint`, `non_claims`.

## Fact rules（事实规则）

1. User-facing statements require code + contract + test agreement; anything
   weaker is labeled `partial`/`designed`/`unavailable` explicitly.
2. Contract-vs-implementation disagreements state both sides ("contract requires
   / implementation does") and register the difference; never silently pick one.
3. Dynamic facts owned by `docs/plan/PROGRESS.md` (task/Slice/Gate/lease/campaign
   status) are linked, never copied — enforced by checker rule HB012.
4. Normative facts link canonical sources; implementation facts link source files
   and tests via `sources[]`/`tests[]` — stable symbols, never line numbers.
5. Examples must be actually executable on the documented platform or explicitly
   marked as conceptual.
6. No secret-shaped content of any kind (HB012).
7. Reference pages that can be generated are generated
   (`generate-handbook.mjs`); hand-editing generated pages fails HB010.

## Writing style

Plain Markdown, no build framework. English pages use complete, direct sentences;
Chinese pages are full translations with identical factual content (status and
sources must match their twin — HB003). Tables carry enumerable facts;
explanations stay in prose. Diagrams are used only for real call relations and
must be traceable to the mapped sources.

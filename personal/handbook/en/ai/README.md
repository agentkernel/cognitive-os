---
doc_id: ai.entry
locale: en
kind: navigation
audience: [ai]
generated: false
---

# AI entry point

Compact orientation for AI coding tools (Cursor, Claude Code, Codex, and similar)
working in this repository. Read these five pages before editing anything:

1. [Source-of-truth order](source-of-truth.md) — which document wins when sources
   disagree, and which facts you must never restate from memory.
2. [Code map](code-map.md) — what each crate, app, and package actually does, with
   the real call chains.
3. [Safe editing boundaries](safe-editing.md) — immutable axioms, protected trees,
   lease rules, and the changes you must never make.
4. [Validation commands](validation-commands.md) — what you can run locally on each
   platform and what must route to CI or native Linux.
5. [Docs impact](docs-impact.md) — when a code change obligates a handbook or legacy
   documentation update in the same PR.

Machine-readable companions: [`personal/handbook/_meta/manifest.json`](../../_meta/manifest.json)
(document inventory), [`personal/handbook/_meta/source-map.json`](../../_meta/source-map.json)
(change → docs routing), [`personal/handbook/_meta/source-coverage.json`](../../_meta/source-coverage.json)
(total tree classification), and the repo-root [`llms.txt`](../../../../llms.txt).

Hard rules that survive any summary: the Rust daemon is the only authority writer;
probabilistic components produce candidates only; secrets never enter argv, config,
SQLite, logs, tests, or evidence; never read or cite `History/`; never copy dynamic
status from `docs/plan/PROGRESS.md`.

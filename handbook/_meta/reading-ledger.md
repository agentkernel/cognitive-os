---
doc_id: meta.reading-ledger
locale: en
kind: meta
audience: [developer, ai]
generated: false
---

# Reading ledger

（locale-neutral 记录：手册创作时的全量阅读台账摘要。机器细节见
[`source-coverage.json`](./source-coverage.json) 与 [`source-set.json`](./source-set.json)。）

## Baselines

- **Reading baseline**: commit `9fbd3904a1f8e0893fcb7d8d2b434e636d546e8c`
  (`origin/main` at audit start) — 1,148 tracked paths, every path classified.
- **Increment**: `9fbd390..6b637a5194dc56b568d2be25bb9ceb3bb24a0f72` (P9-T04
  merge, PR #199, 31 paths) — read in full before authoring completed.
- **Implementation baseline**: `6b637a5194dc56b568d2be25bb9ceb3bb24a0f72`,
  recorded in [`source-set.json`](./source-set.json).

## Coverage method

Eleven read-only audit domains (inventory; kernel core; store/SQLite; runtime
execution; server/CLI/operations; TypeScript clients; specs/conformance
contracts; tests/tools/CI; product/architecture/ADRs; governance/plan/evidence;
checkpoint set) each produced path ledgers, entry points, real call chains,
user-visible capabilities, limits, and test evidence; findings were
cross-verified against Git object reads (`git show`/`git grep` at the baseline
commit) before entering any handbook page.

## Read in full

All live first-party tracked text: `crates/**`, `apps/**`, `packages/**`,
`specs/**`, `conformance/**`, `tests/**`, `tools/**`, `deploy/**`, `scripts/**`,
`.github/**`, `.cursor/rules/*`, `docs/**` (governance, plan, product,
architecture, ADRs, standards, checkpoints, prompts, evaluation, traceability,
legal, research, clients/platforms stubs), root manifests/readmes/whitepaper/
reviews/RFC, generated binding trees (sampled for shape + digest-verified as
generator output), and golden fixtures.

## Excluded, with reasons

| Exclusion | Reason |
|---|---|
| `History/**` | frozen archive; reading/citing forbidden by governance |
| `Cargo.lock`, `pnpm-lock.yaml` | dependency lock payloads without documentation semantics |
| `personal-blog/` | separate repository; never part of this tree |
| Binary/vendored payloads | none tracked at the baseline (verified: text-only tree) |

## Honesty notes

- Facts that changed between the two baselines were re-checked against the
  increment (campaign modules, provider proxy/server touches, Pi client
  correlation changes, closure bookkeeping).
- Where audits disagreed with each other, the Git object read won and the
  discrepancy was resolved before authoring (recorded examples: schema counts,
  scheduler tick behavior, task-route fallback status, CLI usage gaps).

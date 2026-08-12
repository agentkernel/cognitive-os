---
doc_id: meta.sync-policy
locale: en
kind: meta
audience: [developer, ai]
generated: false
---

# Handbook synchronization policy

（locale-neutral 元规范。）How the handbook stays true as the repository evolves.

## Obligations on every change

1. **Route the impact**: match changed paths against
   [`source-map.json`](./source-map.json). Mapped hand-written pages are updated
   and re-fingerprinted (`node tools/src/fill-handbook-fingerprints.mjs`); mapped
   generated pages are regenerated (`node tools/src/generate-handbook.mjs`).
2. **Classify new files**: any new tracked file must match a
   [`source-coverage.json`](./source-coverage.json) rule (HB009) — either an
   owning-doc rule or an excluded category with a reason.
3. **User-visible changes** (CLI, config, errors, security, install, recovery)
   update the user + reference trees; **architectural changes** (data, protocol,
   authority, environments) update the developer + AI trees.
4. **Same task, same PR**: affected handbook pages ship with the code change. A
   genuinely doc-neutral change records `docs-impact: none — <reason>` in the PR.
5. **Checks**: `node tools/src/check-handbook.mjs`,
   `node tools/src/generate-handbook.mjs --check`, and
   `pnpm run check:consistency` must pass; CI runs them on every PR.

## Enforcement layers

| Layer | Mechanism |
|---|---|
| Editor/AI guidance | always-applied rule `.cursor/rules/20-cognitiveos-personal-handbook-sync.mdc` (adapter only; this file owns the policy) |
| Non-Cursor AI tools | root `llms.txt` + [`handbook/en/ai/README.md`](../en/ai/README.md) |
| Machine gate | `check-handbook.mjs` rules HB001–HB015 (manifest, pairing, links, sources, symbols, fingerprints, coverage, generated equality, forbidden content, source-set reproducibility) |
| Task-closure gate | `check-handbook.mjs --diff-base <rev>` proves legacy docs changed only on the allowlist ([`legacy-change-allowlist.json`](./legacy-change-allowlist.json)) |

## Failure semantics

A red handbook check is a build failure equal to any other: fix the page, the
mapping, or the generator input — never bypass, and never "fix" a canonical
source to match documentation. Checker changes themselves require rerunning the
negative fixtures (`tools/test/handbook-check.test.mjs`).

## Baseline advancement

`source-set.json` records the implementation baseline revision the content was
authored against. Advancing it is a deliberate act: read the increment, update
affected pages, regenerate, refresh fingerprints, and record the new revision in
the same PR.

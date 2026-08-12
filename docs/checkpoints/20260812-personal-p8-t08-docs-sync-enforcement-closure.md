# P8-T08 docs-sync enforcement — final acceptance and closure

- Status: task complete; governance/tooling-only delivery
- Task: `P8-T08` (enforce documentation-system synchronization before every commit, push, and merge)
- Branch: `personal/P8-T08-docs-sync-enforcement` (upstream `origin/personal/P8-T08-docs-sync-enforcement`)
- PR: [#203](https://github.com/agentkernel/cognitive-os/pull/203) — Draft until this closure, then ready/merged
- Heads: registration `06d703f`, implementation `539922b`, closure = this commit; base `d514e8ac` (`main` after P8-T07)
- Change class: repo-governance standard update (docs-sync-contract v0.1 → v0.2) + tooling; **normative surface unchanged** (no `specs/`, `conformance/`, registry, schema, transition, or vector change). Owner decision: explicit user directive of 2026-08-12. PERS-PR-031.

## 1. Delivered

- `tools/src/docs-sync-gate.mjs`: conditional gate with `--staged` (pre-commit),
  `--push` (pre-push; merge-base with `@{upstream}` or `origin/main`), and
  `--range` (default `origin/main...HEAD`) modes. Routes changed paths through
  `handbook/_meta/source-map.json`; fast-skips documentation-irrelevant change
  sets; runs the handbook check set (`check-handbook.mjs` +
  `generate-handbook.mjs --check`) on any hit; **fails closed** when mapped
  implementation sources change without any handbook update in the same change
  set. Sole escape: `DOCS_IMPACT_NONE="<concrete reason>"` (blank/trivial
  reasons rejected; the echoed reason must be recorded in the commit/PR).
  Degrades safely on revisions without the handbook system.
- Repo-tracked `.githooks/pre-commit` + `.githooks/pre-push` (mode 100755, thin
  `sh` wrappers), `pnpm run hooks:install` (one-time per clone:
  `git config core.hooksPath .githooks`), `pnpm run check:docs-sync` aggregate.
- Canonical obligations: `docs/standards/docs-sync-contract.md` v0.2 — §2
  handbook block binding **all five change classes** with enforced timing
  (before commit/push/merge), §5 items 16–17 registering the CI red light and
  the local gate, §6 author-checklist item.
- Adapter propagation: `.cursor/rules/10-…` checkpoint/ready-flip obligation
  bullet; `.cursor/rules/20-…` mandatory-timing items;
  `AGENTS.md` checkpoint-protocol bullet + closure-protocol step 3 wording.
- Handbook sync (bilingual, fingerprints refreshed): `_meta/sync-policy.md`
  (obligation timing + enforcement layers), `ai/docs-impact`,
  `ai/validation-commands`, `developer/contributing-workflow`,
  `developer/repository-map`; `_meta/source-coverage.json` gains the
  `.githooks/**` rule, `_meta/source-map.json` gains the
  `docs-sync-enforcement` routing rule, `_meta/legacy-change-allowlist.json`
  regenerated for this task's allowed legacy surface.

## 2. Acceptance mapping

| Acceptance item | Disposition |
|---|---|
| Sync obligation enforced before commit | `.githooks/pre-commit` → gate `--staged`; fail-closed verdict proven by focused tests and by the gate catching this task's own in-flight drift (package.json script additions flagged against 3 fingerprinted page pairs) |
| Sync obligation enforced before push | `.githooks/pre-push` → gate `--push` over the outgoing range |
| Sync obligation enforced before merge | CI `verify` handbook step (unconditional `check-handbook` + generator `--check`) + contract §5 item 16; ready-flip obligations in rule 10/20 and AGENTS closure protocol |
| Canonical constraint, not advice | docs-sync-contract v0.2 §2 (all change classes) + §5 + §6; adapters only reference it |
| Honest no-impact path | explicit `DOCS_IMPACT_NONE` reason, echoed and required in the commit/PR record; blank/trivial rejected (negative test) |
| Focused negatives | 7 gate tests: skip/check/fail/acknowledged/blank-reason/handbook-plus-source/adapter-surface (tools suite 56/56) |
| Real-repo behavior | smokes: unrelated range fast-skips; handbook-bearing range runs the full check set; staged self-run green after sync |
| No second workflow/fact source | obligation semantics live only in the contract; rules/AGENTS/handbook pages point at it |

## 3. Validation record

| Check | Result |
|---|---|
| `node --test tools/test/docs-sync-gate.test.mjs` | pass 7/7 |
| `pnpm --filter @cognitiveos/repo-tools run test` | pass 56/56 |
| `pnpm --filter @cognitiveos/repo-tools run build` (syntax incl. gate) | pass |
| Gate self-run on its own staged change set (`--staged`) | pass (4 rules routed, check set green) |
| `node tools/src/check-handbook.mjs` | pass (54×2 docs, 9 generated families) |
| `node tools/src/generate-handbook.mjs --check` | pass (18 pages byte-identical) |
| `pnpm run check:consistency` | pass |
| `git diff --check` / `--cached --check` | pass |
| Required Ubuntu/Windows CI at the merge head | gates the merge via PR #203 checks |
| Rust build/test on local Windows GNU | not-run (registered unsupported host; CI covers) |

## 4. Non-claims

- The gate enforces synchronization discipline; it does not prove prose
  accuracy beyond the fingerprint/coverage/generated checks it runs.
- Git cannot force hooks on clones; `hooks:install` is per-clone opt-in. CI
  remains the unconditional backstop for every PR.
- No Gate, contract, release, or Profile surface is created or altered.

## 5. Next action

None for this task after merge/lease/branch/main reconciliation. Owner/clones:
run `pnpm run hooks:install` once per working copy to activate the local gate.

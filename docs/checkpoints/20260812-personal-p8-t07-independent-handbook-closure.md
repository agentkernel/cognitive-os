# P8-T07 independent bilingual handbook — final acceptance and closure

- Status: task complete; documentation/tooling-only delivery
- Task: `P8-T07` (independent bilingual handbook system)
- Branch: `personal/P8-T07-independent-handbook` (upstream `origin/personal/P8-T07-independent-handbook`)
- PR: [#202](https://github.com/agentkernel/cognitive-os/pull/202) — Draft until this closure, then ready/merged
- Content head: `f17f558` (registration head `b4d3b83`, first closure head `859ddc6`); final head follows the `origin/main` merge described below
- Reading baseline: `9fbd3904a1f8e0893fcb7d8d2b434e636d546e8c`; increment 1 `9fbd390..6b637a5` (P9-T04) read before authoring; increment 2 `6b637a5..5dd7003` (P2-T09 execution-readiness projection, P7-T07 Windows install surface) read after the concurrent merges landed on `main` — the fingerprint gate flagged exactly the affected pages (9 document pairs), which were updated and regenerated in the same delivery; implementation baseline `5dd7003` (recorded in `handbook/_meta/source-set.json`, 1,158 tracked entries, digest `sha256:b8d768f2…bcb179`)
- Change class: product-semantic task registration + documentation/tooling implementation; **normative surface unchanged** (no `specs/`, `conformance/`, registry, schema, transition, or vector change). PERS-PR-031.

## 1. Delivered

- `handbook/` bilingual system: 54 documents × 2 locales (en, zh-CN) — user (11),
  developer (19), reference (12), AI (6), navigation/root (2 + root README), plus
  5 locale-neutral `_meta` documents (doc standards, sync policy, generation
  spec, reading ledger, glossary).
- Machine model: `manifest.json`, `handbook-frontmatter.schema.json` (2020-12),
  `source-map.json` (change→docs routing), `source-coverage.json` (total tracked
  tree classification), per-page source fingerprints (content SHA-256 + stable
  symbols), `source-set.json` baseline record, 4 annotation files with
  bidirectional anti-rot checks, `legacy-change-allowlist.json`.
- Generated references (9 families × 2 locales, byte-gated): cognitive CLI,
  admin CLI, HTTP API, errors (55 codes), config files, env vars, transitions
  (5 domains), schemas (74), native tool catalog.
- Tooling: `tools/src/handbook-lib.mjs`, `check-handbook.mjs` (rules
  HB001–HB015 + `--diff-base` legacy integrity), `generate-handbook.mjs`
  (`--check` drift gate), `fill-handbook-fingerprints.mjs`;
  `tools/test/handbook-check.test.mjs` (15 tests, one per mandated negative).
- Sync surfaces: `.cursor/rules/20-cognitiveos-personal-handbook-sync.mdc`
  (always-applied adapter), root `llms.txt`, one AGENTS.md pointer, root
  `check:handbook` script, named CI step in `verify`.

## 2. Acceptance mapping

| Acceptance item | Disposition |
|---|---|
| Full live first-party reading ledger + explicit exclusions | `handbook/_meta/reading-ledger.md` + `source-coverage.json`; exclusions: `History/**` (governance), lock payloads (reasons recorded) |
| Three reader entries with complete navigation | root `handbook/README.md` + per-locale user/developer/reference/AI trees; AI entry also via `llms.txt` |
| Capabilities traceable to sources/contracts/tests | every fact page carries `sources[]`+symbols, `contracts[]`, `tests[]`, fingerprint; checker-enforced (HB006/007/008) |
| Uncertain facts labeled, not guessed | `status` enum enforced (HB002/HB011); execution-chain gaps, backup absence, R0/R2/R3 gaps, CLI usage quirks explicitly recorded |
| source map / coverage map / fingerprint / generation runnable | all four run locally + in CI (`check:handbook`, generator `--check`) |
| Sync rule effective | rule 20 (<50 lines, adapter-only) + sync-policy.md + docs-impact pages |
| Anti-drift negatives fail / positives pass | 15/15 focused negative fixtures green (missing source, stale digest, broken link, unmapped file, illegal status, generated drift, missing locale, secret-shaped, dynamic-status copy, History link, …); full green run on the real tree |
| Checks pass | see §3 |
| Existing docs byte-identical except allowed bookkeeping | `check-handbook --diff-base 6b637a5` green against `legacy-change-allowlist.json` (5 plan/governance files, AGENTS.md pointer, tools/package.json, this checkpoint) |
| No other windows' files touched | main workspace `D:\agent-kernel` untouched (its branch/untracked state verified before and after); all work in worktree `D:\agent-kernel-worktrees\P8-T07-handbook` |
| Required CI at exact revision | see §3 |
| Formal acceptance mapping + final handoff + PR/lease/branch/main closure | this document + PROGRESS/plan/lanes closure edits + deterministic merge sequence |

## 3. Validation record

| Check | Result |
|---|---|
| `node tools/src/check-handbook.mjs` | pass (54×2 docs, 9 generated families, coverage/link/fingerprint/status/secret) |
| `node tools/src/generate-handbook.mjs --check` | pass (18 pages byte-identical) |
| `node tools/src/check-handbook.mjs --diff-base 6b637a5…` | pass (legacy integrity) |
| `pnpm --filter @cognitiveos/repo-tools run test` | pass 49/49 (34 pre-existing + 15 handbook negatives) |
| `pnpm --filter @cognitiveos/repo-tools run build` (syntax) | pass |
| `pnpm run check:consistency` | pass (275 requirements, 55 codes, 74 schemas, 89 vectors, leases) |
| `git diff --check` / `--cached --check` | pass |
| Required Ubuntu/Windows CI on content head `f17f558` | run `31572262460`: every pre-existing step green (TS, Rust build/test/clippy/fmt, codegen drift, consistency, traceability); the new handbook step failed only on HB013 `git ls-tree` against the baseline object absent from the shallow CI clone — fixed in `859ddc6` (HB013 skips digest reproducibility with a notice when the revision is unavailable; full clones still verify) |
| Required Ubuntu/Windows CI on first closure head `859ddc6` | dispatch run `31573503885` passed both platforms, including the handbook step |
| Post-merge drift pass (`origin/main` `5dd7003` merged in) | fingerprint gate listed 9 drifted document pairs + 2 generated families; pages updated (Windows Credential Manager backend, Windows install surface, execution-readiness projection), references regenerated, source-set re-recorded; checker + generator `--check` + `--diff-base 5dd7003` green |
| Required Ubuntu/Windows CI on the final head | gates the merge via PR #202 checks |
| Rust build/test on local Windows GNU | not-run (registered unsupported host; CI covers) |

## 4. Non-claims

- The handbook is an informative derived layer: it creates no task, Gate,
  contract, release, Profile, or benefit facts, and copies no dynamic
  `PROGRESS.md`/Gate status (checker-enforced).
- Capability labels are code+contract+test judgments at the recorded baseline,
  not the formal plan's task states.
- No Windows/macOS product support, no execution-chain completion, and no
  production release/signing claim is introduced by any handbook statement.

## 5. Next action

None for this task after merge/lease/branch/main reconciliation. Future source
changes follow `.cursor/rules/20-cognitiveos-personal-handbook-sync.mdc` +
`handbook/_meta/sync-policy.md`; baseline advancement follows
`_meta/generation.md` §2.

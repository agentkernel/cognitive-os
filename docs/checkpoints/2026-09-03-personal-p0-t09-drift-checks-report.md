# P0-T09/D01 — running report (machine drift checks)

- Task / slice: formal task `P0-T09` (Phase 0), Delivery Slice `P0-T09/D01`
- Lease: `lease/personal/P0-T09/drift-checks` (Lane-CFR tool surface + handbook `_meta`)
- Branch: `personal/P0-T09-drift-checks` (worktree `D:\agent-kernel-wt-p0-t09`); implemented on top of
  `DOC-P13-DRIFT-FIX` (PR #309) while GitHub was unreachable from this host, then rebased onto
  `origin/main` after that PR merged (rebase recorded below)
- Environment for every local unit: `DEV-WIN-GNU-01` (Windows PowerShell 5.1; Node tooling only; no Rust link)
- Claim ceiling: `hypothesis`. Tooling/handbook evidence only — no Gate, release, Profile, T15 or Windows-support
  claim. `not-run` is never pass.
- Reporting rule: `TEST-REPORT-INCREMENTAL-01` — append on completion; append-only.

## 1. What changed (per `P0-T09/D01` slice row)

| Item | Implementation | Focused negative fixture |
|---|---|---|
| (1) tracked-only link/path checks | `tools/src/lib.mjs`: `loadTrackedPaths()` (`git ls-files -z` → files + directories) and `isTrackedPath()`; `listMarkdownFiles(tracked)` skips untracked local Markdown. `tools/src/check-consistency.mjs`: fails closed with `TRACKED_PATHS_UNAVAILABLE` outside a Git checkout; markdown links, `owner_spec`, matrix paths, trace `sources`, project-scope paths all decided by the tracked index; message suffix `(exists locally but is not tracked by Git)`. `tools/src/check-agent-rules.mjs`: tracked-only for ordinary paths; local-only editor assets (`LOCAL_ONLY_PREFIXES`) stay filesystem-checked (warn when absent / strict when present); outside a Git checkout falls back to the filesystem and labels `checked.pathExistence` + CLI output | `tools/test/check.test.mjs` "a committed document linking an untracked local file fails the tracked-only link check" (creates a real untracked file, override-injects a link into `docs/plan/plan.md`, expects the suffix); `tools/test/check-agent-rules.test.mjs` "inside a Git checkout … untracked-but-present file fails; passes only once tracked" (`git init` fixture) + "outside a Git checkout … falls back … labels that mode" |
| (2) Phase 13 build-order edge set | `check-consistency.mjs` §7: extracts the first ```mermaid fence of the formal plan section `### Phase 13 - Personal 2.0.0 completion` and of the index section `### Phase 13 build order`, parses `A --> B` / `A -.-> B` edges (ids normalized by stripping `P\d+`), and fails `BUILD_ORDER_EDGE_MISSING` / `BUILD_ORDER_EDGE_EXTRA` / `BUILD_ORDER_GRAPH_MISSING` / `BUILD_ORDER_GRAPH_EMPTY`. Nothing is hard-coded; the formal plan is authoritative | `check.test.mjs`: "edge sets must match" (drop `P13T05 --> P13T13`, add `P13T09 --> P13T13`), "distinguishes dashed from solid edges" (`T06 -.-> T07` → solid in the formal plan), "fails closed when the dev-prep index graph disappears" (heading renamed) |
| (3) `installer.rs` source-map route + symbols | `personal/handbook/_meta/source-map.json` rule `pi-official-package-pin`: `personal/crates/cognitive-runtime/src/installer.rs` → `ref.compatibility`, `dev.agent-pi-lifecycle`, `symbols: [OFFICIAL_PI_PACKAGE, OFFICIAL_PI_VERSION]`. New handbook rule **HB016** (`tools/src/handbook-lib.mjs`): a source-map rule with `symbols` must be pinned by every routed hand-written page in every locale. Both pages (4 files) declare the source with those symbols; fingerprints refreshed | `tools/test/handbook-check.test.mjs`: "source-map symbols must be pinned by every routed page (HB016)" (drop symbol on one locale → 1 hit; drop source entry → `missing: source entry`), "renaming a source-map-pinned symbol in the source fails HB007" (+ HB008 drift); `tools/test/docs-sync-gate.test.mjs`: live source map routes `installer.rs` to the two pages and fails closed without a handbook update |

Mapped handbook pages updated (bilingual) per source-map routing of `tools/**` and `_meta/source-map.json`:
`dev.conformance-testing`, `ai.validation-commands`, `meta.sync-policy` (locale-neutral), `ai.docs-impact`,
`ref.compatibility` (new source + fingerprint), `dev.agent-pi-lifecycle` (symbols).

Observation recorded, not claimed: at this revision no **tracked living-doc link** currently points at one of the
owner's untracked local files (`clients/docs/design/opc-2.0/` 14–18, 21–26, `window-c-*.md`,
`docs/plan/p11-plan-review-and-optimization.md`); the tracked mentions are backtick text in `docs/checkpoints/`
reports and links inside `clients/` (outside the scan roots). The new check is therefore preventive today; the
negative fixture proves it detects the class.

## 2. Failure-first evidence

| # | Unit | Instrument | Environment | Revision | Result | Notes |
|---|---|---|---|---|---|---|
| F1 | New `check.test.mjs` tests against the **HEAD** checker (`tools/src/check-consistency.mjs`, `tools/src/lib.mjs` at `12e84b7c`) | `node --test --test-name-pattern="tracked-only link check|build-order" test/check.test.mjs` | `DEV-WIN-GNU-01` | pre-change checker | **fail (expected)** | `# tests 4 / # pass 0 / # fail 4` |
| F2 | Same tests against the new checker | same | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | `# tests 4 / # pass 4 / # fail 0` |
| F3 | New `check-agent-rules.test.mjs` against HEAD `check-agent-rules.mjs` | `node --test test/check-agent-rules.test.mjs` | `DEV-WIN-GNU-01` | pre-change checker | **fail (expected)** | file fails to load (`PATH_EXISTENCE_*` exports absent) — `# pass 0 / # fail 1` |
| F4 | Same against the new checker | same | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | `# tests 8 / # pass 8` |
| F5 | HB016 test against HEAD `handbook-lib.mjs` (byte-exact `git checkout HEAD -- …`) | `node --test --test-name-pattern="HB016|source-map-pinned" tools/test/handbook-check.test.mjs` | `DEV-WIN-GNU-01` | pre-change lib | **fail (expected)** | HB016 test `not ok` (AssertionError); the HB007 pin test already passes on HEAD (HB007 pre-existed) |
| F6 | Same against the new lib | same | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | `# tests 17 / # pass 17` |

Process incident (recorded, resolved): an attempted `git stash push -- <paths>` from `tools/` used a wrong pathspec and
stashed nothing, after which `git stash pop` applied an **unrelated pre-existing stash** (`stash@{0}` "On
personal/P11-T04-employee: 2026-09-02 closure …") into this worktree with conflicts. Recovery: the three task-owned
files were backed up, `git reset --hard HEAD` restored the worktree, the files were restored byte-identically
(`git diff --stat` unchanged: 3 files, +257/−25), and the stash list was left intact (3 entries, nothing dropped).
No stash is used anywhere else in this delivery; failing-first runs use `git checkout HEAD -- <file>` + backup copy.

## 3. Local validation units

| # | Unit | Instrument | Environment | Revision | Result | Notes |
|---|---|---|---|---|---|---|
| U1 | `node tools/src/check-consistency.mjs` | new checker on the current tree | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | `OK (… tracked-only links … leases, and Phase 13 build-order edge set verified)`; edge sets equal after `DOC-P13-DRIFT-FIX` |
| U2 | `node tools/src/check-handbook.mjs` (first run, before page review) | handbook checker | `DEV-WIN-GNU-01` | worktree (pre-commit) | **fail → fixed** | 6× HB008 drift (`ai.docs-impact`, `dev.conformance-testing`, `ref.compatibility` × 2 locales) — exactly the pages routed by the changed sources; HB016 did not fire (pages pin the symbols). Pages reviewed/updated, `fill-handbook-fingerprints` refreshed 6 pages |
| U3 | `node tools/src/check-handbook.mjs` (rerun) + `generate-handbook.mjs --check` | handbook checker + generator | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | `OK (58 documents x 2 locales, 9 generated)`; `18 pages byte-identical` |
| U4 | `node tools/src/check-agent-rules.mjs` | new checker on the current tree | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | `OK (4 rules, 0 commands, 88 path references, 5 local-only warning(s), path existence = git-tracked)` |
| U5 | `node tools/src/gen-matrix.mjs --check`; `git diff --check` | matrix freshness; whitespace | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | `matrix is up to date`; clean |
| U6 | repo-tools build + full test suite | `pnpm --filter @cognitiveos/repo-tools run build`; `node --test test/*.test.mjs` | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass** | build OK; **124/124** (115 existing + 9 new) |
| U7 | docs-sync-contract §5 injection drill | read-only override fixture (`COGNITIVEOS_CONSISTENCY_OVERRIDE_DIR`) + one temporary untracked file, removed afterwards; working tree untouched | `DEV-WIN-GNU-01` | worktree (pre-commit) | **pass (checker fails as intended)** | output in §4 |
| U8 | rebase onto `origin/main@21f34434` (after `DOC-P13-DRIFT-FIX` merged) + ledger edits, full static gate rerun | `git rebase origin/main`; `check-consistency`; `check-handbook`; `generate-handbook --check`; `check-agent-rules`; `gen-matrix --check`; `git diff --check` | `DEV-WIN-GNU-01` | `64732c61` + working tree | **fail → fixed** | rebase clean (1 commit replayed). `check-consistency` first flagged this very report: the pasted drill output contained a numeric drill requirement id, which the living-doc orphan-REQ check (§5a) correctly reported; digits redacted (`REQ-DRILL-NNN`). All other gates green: handbook OK (58×2), generator OK (18), agent-rules OK (`path existence = git-tracked`), matrix fresh, diff clean |

| U9 | commit + push + Draft PR | docs-sync-gate hooks; `git push -u`; `gh pr create --draft` | `DEV-WIN-GNU-01` | `8f0d83b1` (content `64732c61` + ledger `8f0d83b1`) | **pass** | Draft PR [#312](https://github.com/agentkernel/cognitive-os/pull/312); CI run [33678438650](https://github.com/agentkernel/cognitive-os/actions/runs/33678438650) started |
| U10 | clean-worktree CI simulation | `git worktree add --detach D:\agent-kernel-wt-p0-t09-clean 8f0d83b1` (no untracked local assets: `.cursor/commands/`, rules 30/40, `.cursor/mcp.json`, `clients/docs/design/opc-2.0/14…` absent); `pnpm install --frozen-lockfile --offline`; check set; drill; tests; worktree removed | `DEV-WIN-GNU-01` | `8f0d83b1` | **pass** | `check-consistency` OK (tracked-only, build-order edge set); `check-agent-rules` OK (0 failures / 5 local-only warnings, `path existence = git-tracked`); `check-handbook` OK (58×2); `generate-handbook --check` OK (18); `gen-matrix --check` fresh; §5 drill in the clean tree → exit 1 with the same 5 violations (untracked-but-present link flagged with the suffix); repo-tools **124/124** |
| U11 | required CI | `CI-UBUNTU-01` + `CI-WINDOWS-MSVC-01`, run [33678438650](https://github.com/agentkernel/cognitive-os/actions/runs/33678438650) | GitHub Actions | `8f0d83b1` | **pass** | resolve 4s; verify (ubuntu-latest) 3m49s; verify (windows-latest) 18m37s; required-ci 3s — all SUCCESS (includes the new checkers, repo-tools tests, handbook HB016) |

## 4. Injection drill output (docs-sync-contract §5)

Injected: orphan `REQ-DRILL-NNN` (a numeric drill id; digits redacted here so this report itself is not parsed as an
orphan requirement reference — the live checker did flag the report until redacted, U8) + broken link + link to an
untracked-but-present file into `docs/plan/plan.md` (override); removed `P13T05 --> P13T13` and added
`P13T03 --> P13T13` in the dev-prep index graph (override).

```text
exit code: 1

check-consistency: 5 violation(s)

  docs/plan/plan.md
    broken relative link: ../nowhere/missing.md
  docs/plan/plan.md
    broken relative link: ../checkpoints/.p0-t09-drill-untracked-15324.md (exists locally but is not tracked by Git)
  docs/plan/plan.md
    orphan requirement reference: REQ-DRILL-NNN
  personal/docs/architecture/personal-2.0.0-dev-prep-index.md
    BUILD_ORDER_EDGE_MISSING: formal plan edge absent from the dev-prep index graph: T05 --> T13
  personal/docs/architecture/personal-2.0.0-dev-prep-index.md
    BUILD_ORDER_EDGE_EXTRA: dev-prep index edge absent from the formal plan graph (the formal plan is authoritative): T03 --> T13

cleanup: override dir removed; docs/checkpoints/.p0-t09-drill-untracked-15324.md removed
```

Handbook and agent-rule drills are the focused fixtures F1–F6 above (HB016 one-locale drop → red; `git init`
fixture with an untracked-but-present file → red; tracked → green).

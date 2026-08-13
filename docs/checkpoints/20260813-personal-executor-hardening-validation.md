# Executor hardening review repairs — running validation report

- Activity: owner-directed corrective pre-delivery for actionable defects found
  after merged P2-T10/P2-T11
- Change class: `implementation-only`; normative surface unchanged
- Branch: `personal/executor-hardening-review-fixes`
- Base: `origin/main@5f1c97233fa14bc7e6dd16ce544ebbef5766ccd0`
- Environment routing: local `DEV-WIN-GNU-01` is formatting/static only;
  Rust build/test/Clippy goes to GitHub-hosted Ubuntu/Windows while the KVM soak
  is active. Exact native Linux remains `not-run` until the soak closes.
- Evidence ceiling: implementation evidence only; no Gate, release, Profile,
  B01, benchmark, or Agent-benefit claim.

This is the activity's single append-only validation report under
`TEST-REPORT-INCREMENTAL-01`. Each finished unit is appended before the next
validation unit starts. Later entries may supersede an earlier result but never
erase it.

## Repair matrix

1. Workspace mutation no-follow, handle-relative parent/target/staging access;
   Linux active parent swap and Windows reparse negatives.
2. Per-target OS locking around final preimage CAS and handle-relative rename;
   deterministic competitor at the final check/rename interval.
3. Durable original-key mutation attempt/completion receipts; same-postimage
   competitor and post-execution reversion negatives.
4. Durable HTTP attempted/completed state; executor and EffectProtocol restart
   negatives for indeterminate and completed outcomes.
5. Workspace search no-follow handle-relative opens with post-open type/reparse
   verification; active file and directory swap negatives.
6. Exact immutable catalog descriptor equality at validation and every sink;
   every immutable field drifted across all six families.
7. Enumeration-time `maximum_visited_entries` enforcement with an oversized
   directory negative.
8. Streamed write preimage digest plus explicit bounded patch preimage;
   sparse/over-limit negatives.
9. Cleanup failures surface unknown/indeterminate; durable orphan cleanup and
   hostile-residue recovery negatives.
10. Unified-diff old/new `\ No newline at end of file` semantics.
11. Readiness resolves the secret from the already-loaded Provider config
    snapshot; deterministic config-swap fake-SecretStore negative.

## Incremental results

### V-LOCAL-001 — Rust syntax/format pass during implementation

- Instrument: `cargo fmt --all`
- Revision: uncommitted isolated worktree based on
  `5f1c97233fa14bc7e6dd16ce544ebbef5766ccd0`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: rustfmt parsed and formatted the then-current Rust change set;
  exit 0.
- Disposition: non-final because tests and handbook edits followed; final
  `cargo fmt --all -- --check` must run again before checkpoint.

### V-LOCAL-002 — Patch whitespace check during implementation

- Instrument: `git diff --check`
- Revision: uncommitted isolated worktree based on
  `5f1c97233fa14bc7e6dd16ce544ebbef5766ccd0`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0 before subsequent handbook/test additions.
- Disposition: non-final; rerun on the checkpoint candidate.

### V-LOCAL-003 — Final Rust formatting write

- Instrument: `cargo fmt --all`
- Revision: uncommitted checkpoint candidate based on
  `5f1c97233fa14bc7e6dd16ce544ebbef5766ccd0`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0 after all focused Rust negatives and implementation
  edits then present.
- Disposition: formatting only; the no-write `--check` unit follows.

### V-LOCAL-004 — Rust formatting verification

- Instrument: `cargo fmt --all -- --check`
- Revision: uncommitted checkpoint candidate based on
  `5f1c97233fa14bc7e6dd16ce544ebbef5766ccd0`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0.
- Disposition: local non-linking formatting evidence only.

### V-LOCAL-005 — Handbook generation, first attempt

- Instrument: `node tools/src/generate-handbook.mjs`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `not-run`
- Measurement: Node stopped before generator execution with
  `ERR_MODULE_NOT_FOUND` for package `yaml`; no generated output was produced.
- Disposition: install the lockfile-pinned workspace dependencies, then rerun
  the same generator. This is an environment prerequisite miss, not a handbook
  test failure.

### V-LOCAL-006 — Node dependency prerequisite

- Instrument: `pnpm install --frozen-lockfile`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`, pnpm 10.33.2
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: six workspace projects, lockfile unchanged, 10 cached packages
  installed, exit 0.
- Disposition: handbook/tooling commands are now executable.

### V-LOCAL-007 — Handbook regeneration

- Instrument: `node tools/src/generate-handbook.mjs`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: all nine generated reference families regenerated for both
  locales; exit 0. The HTTP reference now carries the snapshot-consistent
  readiness annotations.
- Disposition: refresh hand-written page fingerprints next.

### V-LOCAL-008 — Handbook fingerprint refresh

- Instrument: `node tools/src/fill-handbook-fingerprints.mjs`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: 12 mapped hand-written pages refreshed across both locales;
  exit 0.
- Disposition: run generator byte check and the complete handbook checker.

### V-LOCAL-009 — Generated handbook byte check

- Instrument: `node tools/src/generate-handbook.mjs --check`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 18/18 generated pages
- Outcome: `pass`
- Measurement: all 18 generated pages byte-identical; exit 0.
- Disposition: generated pages were not hand-edited.

### V-LOCAL-010 — Complete handbook integrity check

- Instrument: `node tools/src/check-handbook.mjs`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 54 documents × 2 locales; 9 generated families
- Outcome: `pass`
- Measurement: coverage, links, fingerprints, status, generated equality and
  secret checks all verified; exit 0.
- Disposition: bilingual docs-sync obligations are satisfied at this worktree
  state.

### V-LOCAL-011 — Patch whitespace check

- Instrument: `git diff --check`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0 after implementation, focused negatives, report and
  handbook synchronization.
- Disposition: no whitespace error in the current diff.

### V-LOCAL-012 — Repository-tool syntax build

- Instrument: `pnpm --filter @cognitiveos/repo-tools run build`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`, Node 22
- Started/retained: 17/17 tooling modules
- Outcome: `pass`
- Measurement: every registered `.mjs` entry passed `node --check`; exit 0.
- Disposition: proceed to focused Node tool tests.

### V-LOCAL-013 — Repository-tool test suite

- Instrument: `pnpm --filter @cognitiveos/repo-tools run test`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`, Node 22
- Started/retained: 58/58 tests
- Outcome: `pass`
- Measurement: 58 passed, 0 failed/cancelled/skipped/todo; duration
  55.078 s; exit 0. Includes docs-sync and handbook failure-first fixtures plus
  current-tree consistency.
- Disposition: Node tooling has no observed regression.

### V-LOCAL-014 — Repository consistency

- Instrument: `pnpm run check:consistency`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: 275 requirements, 55 error codes, 74 schemas and 89 vectors,
  plus links, traceability, plan/Gates, workflow and lease rules verified; exit
  0.
- Disposition: no contract, negative, task, or campaign consistency drift.

### V-LOCAL-015 — Locked Cargo metadata

- Instrument: `cargo metadata --locked --no-deps --format-version 1`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: lockfile and workspace package graph resolved without
  compilation or linking; exit 0.
- Disposition: safe local metadata evidence only; Rust type/build validation
  remains routed to supported CI.

### V-LOCAL-016 — Rust formatting recheck

- Instrument: `cargo fmt --all -- --check`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0 after final unused-import cleanup.
- Disposition: formatting remains clean.

### V-LOCAL-017 — Rust formatting after self-review

- Instrument: `cargo fmt --all -- --check`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `fail`
- Measurement: rustfmt requested line wrapping in three newly added
  key-bound-receipt checks; exit 1. No compile/link was attempted.
- Disposition: apply rustfmt and rerun; this is a local formatting defect.

### V-LOCAL-018 — Rustfmt self-review correction

- Instrument: `cargo fmt --all`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: rustfmt applied the three requested line-wrap corrections;
  exit 0.
- Disposition: rerun the no-write formatting check.

### V-LOCAL-019 — Corrected Rust formatting verification

- Instrument: `cargo fmt --all -- --check`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0.
- Disposition: the formatting failure in V-LOCAL-017 is resolved.

### V-LOCAL-020 — Post-review fingerprint refresh

- Instrument: `node tools/src/fill-handbook-fingerprints.mjs`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: four readiness-mapped bilingual pages refreshed; exit 0.
- Disposition: rerun the complete handbook checker on the checkpoint candidate.

### V-LOCAL-021 — Post-review handbook check

- Instrument: `node tools/src/check-handbook.mjs`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 54 documents × 2 locales; 9 generated families
- Outcome: `pass`
- Measurement: complete coverage/link/fingerprint/status/secret check set
  passed; exit 0.
- Disposition: handbook remains synchronized after self-review corrections.

### V-LOCAL-022 — Final self-review formatting write

- Instrument: `cargo fmt --all`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0 after root-handle anchoring, enumeration-budget,
  key-bound-receipt and content-retention self-review corrections.
- Disposition: run the no-write formatting check before staging.

### V-LOCAL-023 — Final self-review formatting verification

- Instrument: `cargo fmt --all -- --check`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0.
- Disposition: final local Rust formatting gate is green.

### V-LOCAL-024 — Candidate fingerprint convergence

- Instrument: `node tools/src/fill-handbook-fingerprints.mjs`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: 0 pages required updates; exit 0.
- Disposition: mapped handbook fingerprints were already current.

### V-LOCAL-025 — Candidate generated-page check

- Instrument: `node tools/src/generate-handbook.mjs --check`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 18/18 pages
- Outcome: `pass`
- Measurement: all generated pages byte-identical; exit 0.
- Disposition: candidate generated references are converged.

### V-LOCAL-026 — Candidate handbook integrity

- Instrument: `node tools/src/check-handbook.mjs`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 54 documents × 2 locales; 9 generated families
- Outcome: `pass`
- Measurement: full coverage/link/fingerprint/status/secret check set passed;
  exit 0.
- Disposition: final bilingual handbook candidate is green.

### V-LOCAL-027 — Candidate repository consistency

- Instrument: `pnpm run check:consistency`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: 275 requirements, 55 error codes, 74 schemas, 89 vectors and
  governance/link/trace/lease invariants verified; exit 0.
- Disposition: candidate has no observed static consistency drift.

### V-LOCAL-028 — Candidate patch whitespace

- Instrument: `git diff --check`
- Revision: uncommitted checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0.
- Disposition: checkpoint candidate contains no whitespace errors.

### V-LOCAL-029 — Staged docs-sync gate

- Instrument: `node tools/src/docs-sync-gate.mjs --staged`
- Revision: staged checkpoint candidate
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: daemon-http (2 paths), scheduler-execution (11 paths) and
  handbook-itself (19 paths) routed to the required documents; complete
  handbook and 18-page generated-byte checks passed; exit 0.
- Disposition: no `DOCS_IMPACT_NONE` escape was used.

### V-CI-001 — Required Ubuntu/Windows matrix, checkpoint 0313f05

- Instrument: GitHub Actions `CI`, run `31658405942`
- Revision: `0313f053c71522bd657136a30d4b9156ac840768`
- Environment: `CI-UBUNTU-01` job `94317836782`;
  `CI-WINDOWS-MSVC-01` job `94317836566`
- Started/retained: 2/2 jobs
- Outcome: `fail`
- Measurement: Ubuntu failed at `Build Rust workspace`; downstream Rust and
  repository checks were skipped. Windows was retained and also failed at
  `Build Rust workspace`. No job or failure was discarded or rerun yet.
- Disposition: inspect both failed build logs, repair only branch-owned
  compile defects, then push a new immutable checkpoint. This run is not
  implementation-pass evidence.

#### V-CI-001 diagnostic addendum

Both jobs failed before compilation with the same Cargo error: `Cargo.lock`
would need an update while `--locked` was active (exit 101). The run therefore
contains no Rust type/test result. The branch added `cap-std`/`cap-fs-ext`; the
recovery action is to regenerate the lockfile with the pinned toolchain, verify
full dependency metadata under `--locked`, and push a new checkpoint.

### V-LOCAL-030 — Full Cargo metadata regeneration

- Instrument: `cargo metadata --format-version 1`
- Revision: dirty recovery worktree after `0313f053c71522bd657136a30d4b9156ac840768`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: full transitive metadata resolved and the lockfile was updated;
  exit 0. No crate compilation or linking occurred.
- Disposition: verify the regenerated graph with `--locked`.

### V-LOCAL-031 — Full locked Cargo metadata verification

- Instrument: `cargo metadata --locked --format-version 1`
- Revision: dirty recovery worktree after `0313f053c71522bd657136a30d4b9156ac840768`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: full transitive graph resolved under `--locked`; exit 0. No
  compilation or linking occurred.
- Disposition: the matrix's pre-compilation lockfile failure is repaired.

### V-LOCAL-032 — Lockfile-recovery docs-sync gate

- Instrument: `node tools/src/docs-sync-gate.mjs --staged`
- Revision: staged recovery candidate after `0313f053c71522bd657136a30d4b9156ac840768`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 2/2 changed paths
- Outcome: `pass`
- Measurement: `Cargo.lock` plus this running report were correctly classified
  as documentation-neutral; exit 0.
- Disposition: no mapped behavior source changed in this recovery checkpoint.

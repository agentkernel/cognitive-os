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

### V-CI-002 — Required Ubuntu/Windows matrix, checkpoint fd88e20

- Instrument: GitHub Actions `CI`, run `31658734870`
- Revision: `fd88e20520b292c34f86e49136165f1828a397a7`
- Environment: `CI-UBUNTU-01` job `94318829693`;
  `CI-WINDOWS-MSVC-01` job `94318829664`
- Started/retained: 2/2 jobs
- Outcome: `fail`
- Measurement: the lockfile gate was repaired and both jobs reached Rust
  compilation; both then failed in `state.rs` at the same ambiguous
  `by_ref` method resolution. Rust tests and downstream checks were skipped.
- Disposition: disambiguate the standard `Read::by_ref` call, run local
  non-linking guards, and push a new immutable checkpoint.

#### V-CI-002 diagnostic addendum

The complete Windows log exposed two additional errors at the same compile
step: stable Rust rejects `MetadataExt::volume_serial_number/file_index` as
`windows_by_handle`. The repair uses the safe cross-platform `same-file`
handle identity abstraction; no unsafe block or platform claim is introduced.

### V-LOCAL-033 — Safe handle-identity dependency resolution

- Instrument: `cargo add same-file --package kernel-server`
- Revision: dirty recovery worktree after `fd88e20520b292c34f86e49136165f1828a397a7`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: added latest compatible `same-file` 1.0.6; exit 0.
- Disposition: use its safe `Handle::from_file` equality/hash rather than
  unstable Windows metadata or repository-forbidden unsafe Win32 calls.

### V-LOCAL-034 — Compile-fix formatting

- Instrument: `cargo fmt --all -- --check`
- Revision: dirty recovery worktree after `fd88e20520b292c34f86e49136165f1828a397a7`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0.
- Disposition: compile-fix Rust formatting is clean.

### V-LOCAL-035 — Compile-fix locked metadata

- Instrument: `cargo metadata --locked --format-version 1`
- Revision: dirty recovery worktree after `fd88e20520b292c34f86e49136165f1828a397a7`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: full transitive graph resolved under `--locked`; exit 0.
- Disposition: checkpoint may return to supported-CI compilation.

### V-LOCAL-036 — Compile-fix repository consistency

- Instrument: `pnpm run check:consistency`
- Revision: dirty recovery worktree after `fd88e20520b292c34f86e49136165f1828a397a7`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: all registered contract, plan, link, trace and lease checks
  passed; exit 0.
- Disposition: no static consistency regression from the compile repair.

### V-LOCAL-037 — Compile-fix handbook integrity

- Instrument: `node tools/src/check-handbook.mjs`
- Revision: dirty recovery worktree after `fd88e20520b292c34f86e49136165f1828a397a7`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 54 documents × 2 locales; 9 generated families
- Outcome: `pass`
- Measurement: complete handbook check set passed; exit 0.
- Disposition: mapped documentation remains synchronized.

### V-LOCAL-038 — Compile-fix docs-sync gate, first attempt

- Instrument: `node tools/src/docs-sync-gate.mjs --staged`
- Revision: staged recovery candidate after `fd88e20520b292c34f86e49136165f1828a397a7`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `fail`
- Measurement: the gate mapped three scheduler-execution paths and required
  either another handbook edit or a concrete documentation-neutral
  acknowledgement; exit 1.
- Disposition: this delta only disambiguates a trait method and replaces
  unstable Windows identity access with a safe equivalent already described
  as handle identity. Rerun with that exact `DOCS_IMPACT_NONE` reason and
  record it in the commit/PR.

### V-LOCAL-039 — Compile-fix docs-sync gate with concrete acknowledgement

- Instrument: `node tools/src/docs-sync-gate.mjs --staged`
- Revision: staged recovery candidate after `fd88e20520b292c34f86e49136165f1828a397a7`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: gate accepted the concrete reason: “Compile-only repair:
  disambiguates `Read::by_ref` and replaces unstable Windows metadata calls
  with the safe `same-file` handle identity already documented; runtime
  semantics and mapped handbook facts are unchanged.” Full handbook and
  generated-byte checks also passed; exit 0.
- Disposition: record the identical reason in the commit and PR.

### V-CI-003 — Required Ubuntu/Windows matrix, checkpoint 49d9d45

- Instrument: GitHub Actions `CI`, run `31659272508`
- Revision: `49d9d4552d28e8f8b6cd2de91557244e88c66e79`
- Environment: `CI-UBUNTU-01` job `94320455069`;
  `CI-WINDOWS-MSVC-01` job `94320455104`
- Started/retained: 2/2 jobs
- Outcome: `fail`
- Measurement: both jobs passed workspace compilation and failed during the
  Rust workspace test step. Ubuntu stopped after 1m55s; Windows retained its
  full 9m15s run before failing. Downstream Clippy/repository checks were
  skipped.
- Disposition: inspect both complete test logs and repair the branch-owned
  cross-platform focused negatives without weakening them.

#### V-CI-003 diagnostic addendum

Both jobs reported the same four failures. Two new HTTP EffectProtocol restart
fixtures used a timestamp after the existing authorization lease and were
correctly rejected as stale; the fixture now uses the established grant time.
One pre-existing workspace-read assertion still expected the former test-only
16-byte descriptor drift and is corrected to the immutable catalog output.
Finally, a linked parent failed closed as a `PortFailure`; mutation dispatch
now classifies that non-I/O path refusal explicitly as `NotExecuted`, while
preserving the no-write assertion. The negatives are retained, not weakened.

### V-LOCAL-040 — Focused-test repair formatting, first attempt

- Instrument: `cargo fmt --all -- --check`
- Revision: dirty recovery worktree after `49d9d4552d28e8f8b6cd2de91557244e88c66e79`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `fail`
- Measurement: rustfmt requested one expression reflow in the linked-parent
  refusal branch; exit 1.
- Disposition: apply rustfmt and rerun.

### V-LOCAL-041 — Focused-test repair rustfmt

- Instrument: `cargo fmt --all`
- Revision: dirty recovery worktree after `49d9d4552d28e8f8b6cd2de91557244e88c66e79`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: one expression reflow applied; exit 0.
- Disposition: rerun the no-write check.

### V-LOCAL-042 — Focused-test repair formatting verification

- Instrument: `cargo fmt --all -- --check`
- Revision: dirty recovery worktree after `49d9d4552d28e8f8b6cd2de91557244e88c66e79`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0.
- Disposition: focused test repairs are formatting-clean.

### V-LOCAL-043 — Focused-test repair consistency

- Instrument: `pnpm run check:consistency`
- Revision: dirty recovery worktree after `49d9d4552d28e8f8b6cd2de91557244e88c66e79`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: complete repository consistency set passed; exit 0.
- Disposition: no static contract/governance drift.

### V-LOCAL-044 — Focused-test repair handbook integrity

- Instrument: `node tools/src/check-handbook.mjs`
- Revision: dirty recovery worktree after `49d9d4552d28e8f8b6cd2de91557244e88c66e79`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 54 documents × 2 locales; 9 generated families
- Outcome: `pass`
- Measurement: complete handbook check set passed; exit 0.
- Disposition: existing mapped wording already describes the retained
  fail-closed behavior.

### V-LOCAL-045 — Focused-test repair docs-sync gate

- Instrument: `node tools/src/docs-sync-gate.mjs --staged`
- Revision: staged recovery candidate after `49d9d4552d28e8f8b6cd2de91557244e88c66e79`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: full handbook/generated checks passed with the concrete reason:
  “Focused CI repair aligns a stale fixture and assertion and classifies the
  already-documented fail-closed linked-parent refusal as `NotExecuted`;
  mapped capability semantics are unchanged.”
- Disposition: record the same reason in the commit and PR.

### V-CI-004 — Required Ubuntu/Windows matrix, checkpoint 46bb496

- Instrument: GitHub Actions `CI`, run `31660187159`
- Revision: `46bb4969e8d870d6bde0703e72420339bae8def7`
- Environment: `CI-UBUNTU-01` job `94323204232`;
  `CI-WINDOWS-MSVC-01` job `94323204241`
- Started/retained: 2/2 jobs
- Outcome: `fail`
- Measurement: Ubuntu passed workspace build and the complete Rust test step,
  then failed Clippy on five branch-owned warnings. Windows passed workspace
  build but retained one platform-specific Rust test failure after 8m56s;
  downstream checks were skipped.
- Disposition: repair the five mechanical Clippy findings and inspect the
  retained Windows-only negative without weakening reparse protection.

#### V-CI-004 diagnostic addendum

Ubuntu's complete Rust test step passed, including every new executor and
readiness negative. Its five Clippy findings are mechanical (`int_plus_one`,
`too_many_arguments`, `needless_return`, `io_other_error`, and
`drop_non_drop`). Windows' only test failure was the pre-existing
`p1_t05_personal_readiness` 2-second bootstrap-secret startup race previously
recorded by P2-T10; all branch-focused Windows tests passed. The next immutable
revision carries only the Clippy repairs; if the unrelated startup flake
repeats, its bounded wait will be repaired rather than ignored.

### V-LOCAL-046 — Clippy repair formatting, first attempt

- Instrument: `cargo fmt --all -- --check`
- Revision: dirty recovery worktree after `46bb4969e8d870d6bde0703e72420339bae8def7`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `fail`
- Measurement: rustfmt requested one indentation correction around
  `io::Error::other`; exit 1.
- Disposition: apply rustfmt and rerun.

### V-LOCAL-047 — Clippy repair rustfmt

- Instrument: `cargo fmt --all`
- Revision: dirty recovery worktree after `46bb4969e8d870d6bde0703e72420339bae8def7`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: one indentation correction applied; exit 0.
- Disposition: rerun the no-write check.

### V-LOCAL-048 — Clippy repair formatting verification

- Instrument: `cargo fmt --all -- --check`
- Revision: dirty recovery worktree after `46bb4969e8d870d6bde0703e72420339bae8def7`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0.
- Disposition: Clippy repairs are formatting-clean.

### V-LOCAL-049 — Clippy repair consistency

- Instrument: `pnpm run check:consistency`
- Revision: dirty recovery worktree after `46bb4969e8d870d6bde0703e72420339bae8def7`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: complete consistency set passed; exit 0.
- Disposition: no static drift.

### V-LOCAL-050 — Clippy repair handbook integrity

- Instrument: `node tools/src/check-handbook.mjs`
- Revision: dirty recovery worktree after `46bb4969e8d870d6bde0703e72420339bae8def7`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 54 documents × 2 locales; 9 generated families
- Outcome: `pass`
- Measurement: complete handbook check set passed; exit 0.
- Disposition: documentation remains converged.

### V-LOCAL-051 — Clippy repair docs-sync gate

- Instrument: `node tools/src/docs-sync-gate.mjs --staged`
- Revision: staged recovery candidate after `46bb4969e8d870d6bde0703e72420339bae8def7`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: full handbook/generated checks passed with the concrete reason:
  “Clippy-only refactors group existing publish arguments and apply equivalent
  standard-library idioms; runtime semantics and mapped handbook facts are
  unchanged.”
- Disposition: record the same reason in the commit and PR.

### V-CI-005 — Required Ubuntu/Windows matrix, checkpoint fdb0ea4

- Instrument: GitHub Actions `CI`, run `31661255206`
- Revision: `fdb0ea4b797a914de0c0c70ee07750018795a093`
- Environment: `CI-UBUNTU-01` job `94326380221`;
  `CI-WINDOWS-MSVC-01` job `94326380431`
- Started/retained: 2/2 jobs
- Outcome: `pass`
- Measurement: both jobs passed TypeScript build/tests, Rust workspace
  build/tests, Clippy with deny-warnings, rustfmt, codegen diff, consistency,
  traceability, handbook/generated drift, conformance report/honesty,
  wrong-implementation self-check, cross-language digest and artifact upload.
  Ubuntu completed in 2m32s; Windows completed in 9m13s.
- Disposition: strongest evidence is `tested-supported-ci`; ordinary CI creates
  no Gate/release/Profile/B01/benchmark claim. Exact native Linux remains
  deferred until the active soak/campaign closes.

## Post-CI independent self-review

The defect-first review of `origin/main...fdb0ea4` found three additional
actionable fail-closed gaps before declaring the change clean:

1. absent/corrupt HTTP durable state still mapped to `NotExecuted`, which could
   erase an unresolved attempt after state loss;
2. state-root `create_dir_all` could create through a planted link before the
   later no-follow open rejected it;
3. mutation receipts were not required to live outside the approved workspace,
   allowing a workspace Tool path to target its own proof store.

The follow-up makes missing HTTP state `Indeterminate`, creates every state-root
component handle-relatively with no-follow semantics, requires mutation state
outside the workspace, moves all focused fixtures to isolated sibling state,
and adds negatives for state loss, linked-root creation, and unsafe receipt
placement.

### V-LOCAL-052 — Self-review repair formatting, first attempt

- Instrument: `cargo fmt --all -- --check`
- Revision: dirty self-review worktree after
  `fdb0ea4b797a914de0c0c70ee07750018795a093`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `fail`
- Measurement: rustfmt requested two test-expression compactions; exit 1.
- Disposition: apply rustfmt and rerun.

### V-LOCAL-053 — Self-review repair rustfmt

- Instrument: `cargo fmt --all`
- Revision: dirty self-review worktree after
  `fdb0ea4b797a914de0c0c70ee07750018795a093`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: two test-expression compactions applied; exit 0.
- Disposition: rerun the no-write check.

### V-LOCAL-054 — Self-review repair formatting verification

- Instrument: `cargo fmt --all -- --check`
- Revision: dirty self-review worktree after
  `fdb0ea4b797a914de0c0c70ee07750018795a093`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0.
- Disposition: self-review repairs are formatting-clean.

### V-LOCAL-055 — Self-review repair consistency

- Instrument: `pnpm run check:consistency`
- Revision: dirty self-review worktree after
  `fdb0ea4b797a914de0c0c70ee07750018795a093`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: complete consistency set passed; exit 0.
- Disposition: no static contract/governance drift.

### V-LOCAL-056 — Self-review handbook fingerprint convergence

- Instrument: `node tools/src/fill-handbook-fingerprints.mjs`
- Revision: dirty self-review worktree after
  `fdb0ea4b797a914de0c0c70ee07750018795a093`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: 0 additional fingerprint updates required; exit 0.
- Disposition: run complete handbook/generated checks.

### V-LOCAL-057 — Self-review generated-page check

- Instrument: `node tools/src/generate-handbook.mjs --check`
- Revision: dirty self-review worktree after
  `fdb0ea4b797a914de0c0c70ee07750018795a093`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 18/18 pages
- Outcome: `pass`
- Measurement: all generated pages byte-identical; exit 0.
- Disposition: generated references remain converged.

### V-LOCAL-058 — Self-review handbook integrity

- Instrument: `node tools/src/check-handbook.mjs`
- Revision: dirty self-review worktree after
  `fdb0ea4b797a914de0c0c70ee07750018795a093`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 54 documents × 2 locales; 9 generated families
- Outcome: `pass`
- Measurement: complete handbook check set passed; exit 0.
- Disposition: bilingual self-review documentation is synchronized.

### V-LOCAL-059 — Self-review staged docs-sync gate

- Instrument: `node tools/src/docs-sync-gate.mjs --staged`
- Revision: staged self-review candidate after
  `fdb0ea4b797a914de0c0c70ee07750018795a093`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: five scheduler-execution paths and six bilingual handbook
  paths routed correctly; full handbook and generated-byte checks passed;
  exit 0.
- Disposition: no docs-impact escape was used.

### V-CI-006 — Required Ubuntu/Windows matrix, checkpoint 868959d

- Instrument: GitHub Actions `CI`, run `31662914498`
- Revision: `868959d6d27c76b44d6dd8f481b6563e376a4409`
- Environment: `CI-UBUNTU-01` job `94331300976`;
  `CI-WINDOWS-MSVC-01` job `94331300911`
- Started/retained: 2/2 jobs
- Outcome: `pass`
- Measurement: both jobs passed the complete required matrix: TypeScript,
  Rust workspace build/tests, Clippy deny-warnings, rustfmt, codegen,
  consistency, traceability, bilingual handbook/generated drift, conformance
  honesty/self-checks, cross-language digest and artifacts. Ubuntu completed
  in 3m12s; Windows completed in 8m38s.
- Disposition: the post-CI self-review repairs are `tested-supported-ci`.
  Ordinary CI creates no Gate/release/Profile/B01/benchmark claim.

### Final-review follow-up

The second defect-first pass found one remaining state-loss path: restaging a
previously seen key after its record disappeared could recreate `Staged` and
launder an unknown/completed attempt into `NotExecuted`. Stable per-key lock
files now act as an independent seen-key witness. If a seen key has no record,
HTTP and mutation staging refuse to recreate it and reconciliation remains
`Indeterminate`. Focused negatives remove a record, restart, attempt to
restage, and prove no second dispatch/non-execution claim.

### V-LOCAL-060 — Final-review follow-up formatting, first attempt

- Instrument: `cargo fmt --all -- --check`
- Revision: dirty final-review worktree after
  `868959d6d27c76b44d6dd8f481b6563e376a4409`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `fail`
- Measurement: rustfmt requested two closure-indentation corrections; exit 1.
- Disposition: apply rustfmt and rerun.

### V-LOCAL-061 — Final-review follow-up rustfmt

- Instrument: `cargo fmt --all`
- Revision: dirty final-review worktree after
  `868959d6d27c76b44d6dd8f481b6563e376a4409`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: two closure-indentation corrections applied; exit 0.
- Disposition: rerun the no-write check.

### V-LOCAL-062 — Final-review follow-up formatting verification

- Instrument: `cargo fmt --all -- --check`
- Revision: dirty final-review worktree after
  `868959d6d27c76b44d6dd8f481b6563e376a4409`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: exit 0.
- Disposition: final-review follow-up is formatting-clean.

### V-LOCAL-063 — Final-review follow-up consistency

- Instrument: `pnpm run check:consistency`
- Revision: dirty final-review worktree after
  `868959d6d27c76b44d6dd8f481b6563e376a4409`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: complete repository consistency set passed; exit 0.
- Disposition: no static drift.

### V-LOCAL-064 — Final-review follow-up handbook integrity

- Instrument: `node tools/src/check-handbook.mjs`
- Revision: dirty final-review worktree after
  `868959d6d27c76b44d6dd8f481b6563e376a4409`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 54 documents × 2 locales; 9 generated families
- Outcome: `pass`
- Measurement: complete handbook check set passed; exit 0.
- Disposition: mapped state-loss wording remains synchronized.

### V-LOCAL-065 — Final-review follow-up docs-sync gate

- Instrument: `node tools/src/docs-sync-gate.mjs --staged`
- Revision: staged final-review candidate after
  `868959d6d27c76b44d6dd8f481b6563e376a4409`
- Environment: `DEV-WIN-GNU-01`
- Started/retained: 1/1
- Outcome: `pass`
- Measurement: full handbook/generated checks passed with the concrete reason:
  “Seen-key witness closes state-loss restaging while preserving the already
  documented rule that missing durable state reconciles `Indeterminate`;
  mapped handbook facts are unchanged.”
- Disposition: record the same reason in the commit and PR.

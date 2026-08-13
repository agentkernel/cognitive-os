# P2-T16 `RegisteredCheckRun` 增量验证报告

- Activity: P2-T16/D01-D04
- Branch: `personal/P2-T16-registered-check-run`
- Dependency base: open Draft PR #210 head
  `b514d278ef4a3daafe9cceeb62ced2dc649d186b`
- Claim ceiling: implementation evidence / non-claim
- Report policy: `TEST-REPORT-INCREMENTAL-01`; 每个完成单元立即追加

## 2026-08-13 — FF-001 `check_id`-only caller boundary

- Instrument:
  `cargo test -p kernel-server
  personal::registered_check::tests::caller_can_request_registered_check_by_check_id_only`
- Started denominator: 1 authored failure-first test.
- Environment: `DEV-WIN-GNU-01`.
- Outcome: **not-run** locally. Repository policy forbids Rust compile/link/test on
  this registered unsupported host.
- Retained observation: the test imports the desired
  `RegisteredCheckRunRequest` and `RegisteredCheckRegistry`; neither production
  symbol exists at the dependency base, so the supported CI checkpoint is
  intentionally expected to fail before implementation.
- Disposition: commit and push this red checkpoint, retain its required
  Ubuntu/Windows CI result, then implement without weakening or deleting the
  test.
- Non-claims: no check dispatched; no Effect, Evidence, verification, Task
  completion, Gate, release, Profile, B01, or evaluation result.

## 2026-08-13 — LOCAL-001 formatting and editor diagnostics

- Instruments: `cargo fmt --all -- --check`; Cursor diagnostics for the two
  edited Rust files.
- Exact worktree base: `b514d278ef4a3daafe9cceeb62ced2dc649d186b`
  plus the uncommitted FF-001 change.
- Outcome: **pass**. Formatting exited 0 and editor diagnostics reported no
  findings.
- Scope: syntax-independent local checks only; no Rust compilation, linking,
  test, or behavior claim.

## 2026-08-13 — LOCAL-002 consistency first attempt

- Instrument: `pnpm run check:consistency`.
- Environment: isolated worktree before dependency installation.
- Outcome: **not-run**. Node stopped before the checker loaded because this new
  worktree had no `node_modules` and could not resolve `ajv`.
- Disposition: install the lockfile-pinned workspace dependencies, then rerun
  the same command; this is an environment prerequisite, not a consistency
  failure.

## 2026-08-13 — LOCAL-003 consistency rerun

- Instrument: `pnpm install --frozen-lockfile`, followed by
  `pnpm run check:consistency`.
- Environment: isolated P2-T16 worktree.
- Outcome: **pass**. The checker verified 275 requirements, 55 error codes,
  74 schemas, 89 vectors, traceability, Personal plan/Gates and leases.
- Scope: governance/static consistency only; FF-001 remains intentionally red
  for supported Rust CI.

## 2026-08-13 — LOCAL-004 checkpoint safeguards

- Instruments: `git diff --check`; staged
  `node tools/src/docs-sync-gate.mjs --staged`.
- Outcome: **pass**. Diff whitespace validation exited 0; the docs-sync gate
  checked six staged paths and found no documentation-relevant production
  behavior because this checkpoint contains only the absent-API test and task
  registration.
- Disposition: retain FF-001 unchanged and create the requested red checkpoint
  commit.

## 2026-08-13 — CI-RED-001 failure-first checkpoint

- Exact revision: `31c089aa99bd54ff7dca20f41b61e3817ea73c49`.
- Instrument: required CI run
  [31716967382](https://github.com/agentkernel/cognitive-os/actions/runs/31716967382),
  `cargo test --workspace --locked -- --test-threads=1`.
- Started/retained denominator: Ubuntu 1/1 job; Windows 1/1 job.
- Outcome: **expected fail** on both platforms at the same boundary. Rust
  `E0432` reports that `RegisteredCheckRegistry` and
  `RegisteredCheckRunRequest` do not exist in
  `personal::registered_check`; each job exits 101 during the Rust test step.
- Control observation: both jobs completed dependency setup, TypeScript
  build/tests and the Rust workspace build before the retained failure.
- Disposition: failure-first condition is now observed on both supported CI
  platforms. Implement the two production symbols and the immutable descriptor
  boundary without changing the test expectation.
- Non-claims: the expected red run is not implementation evidence and does not
  validate dispatch, Effect recovery, Evidence, verification, or completion.

## 2026-08-13 — LOCAL-005 implementation parse/format diagnostics

- Instruments: `cargo fmt --all`; editor diagnostics over all edited Rust
  implementation and test files.
- Worktree parent: exact red checkpoint
  `31c089aa99bd54ff7dca20f41b61e3817ea73c49`.
- Outcome: **pass**. Rustfmt parsed and formatted the new module graph; editor
  diagnostics reported no findings.
- Scope: non-linking local evidence only. Rust behavior remains pending
  supported CI.

## 2026-08-13 — LOCAL-006 post-integration formatting

- Instrument: `cargo fmt --all -- --check`.
- Outcome: **pass** after registry, executor, router, helper, CAS verifier and
  focused-negative integration.
- Scope: formatting/parse only on `DEV-WIN-GNU-01`; no local Rust behavior
  claim.

## 2026-08-13 — LOCAL-007 implementation consistency

- Instrument: `pnpm run check:consistency`.
- Outcome: **pass**; 275 requirements, 55 errors, 74 schemas, 89 vectors,
  traceability, task counts, slices and active lease were consistent.
- Scope: static repository consistency; supported Rust CI remains required.

## 2026-08-13 — DOC-001 generated bilingual references

- Instrument: `node tools/src/generate-handbook.mjs`.
- Outcome: **pass**. All 18 generated pages were regenerated; the native Tool
  catalog now derives its count and includes `native.registered-check.run` in
  both locales, while the HTTP reference reflects the unchanged route set and
  refreshed server-source fingerprint.
- Non-claim: generated presence does not promote support, Gate, release or
  Profile status.

## 2026-08-13 — DOC-002 authored-page fingerprints

- Instrument: `node tools/src/fill-handbook-fingerprints.mjs`.
- Outcome: **pass**. Sixteen source-dependent authored pages across both
  locales received refreshed fingerprints after the registry/router/verifier
  changes.
- Disposition: run byte-identical generator and handbook consistency checks
  after all authored content is finalized.

## 2026-08-13 — DOC-003 generated-page byte gate

- Instrument: `node tools/src/generate-handbook.mjs --check`.
- Outcome: **pass**; all 18 generated pages are byte-identical to generator
  output.

## 2026-08-13 — DOC-004 handbook check first attempt

- Instrument: `node tools/src/check-handbook.mjs`.
- Outcome: **fail** with two `HB006` findings, one per locale. Both point to the
  new integration-test path because the checker intentionally accepts only
  Git-tracked test references and the new file was still untracked.
- Disposition: stage that task-owned test path, then rerun the unchanged
  handbook checker.

## 2026-08-13 — DOC-005 handbook check rerun

- Instrument: unchanged `node tools/src/check-handbook.mjs` after staging the
  new task-owned integration test.
- Outcome: **pass**; 54 documents × 2 locales, nine generated families,
  coverage, links, fingerprints, statuses and secret checks verified.

## 2026-08-13 — LOCAL-008 post-doc consistency

- Instrument: `pnpm run check:consistency`.
- Outcome: **pass** after generated pages, authored bilingual updates,
  fingerprints and expanded exact-path lease.

## 2026-08-13 — LOCAL-009 repository tools

- Instrument: `pnpm --filter @cognitiveos/repo-tools test`.
- Outcome: **pass**, 58/58. This includes consistency failure injection,
  docs-sync routing negatives, handbook HB001-HB015 checks and generated-page
  drift detection.

## 2026-08-13 — LOCAL-010 diff and diagnostics

- Instruments: unstaged/staged `git diff --check`; editor diagnostics over the
  changed Rust app/kernel and handbook generator paths.
- Outcome: **pass**; no whitespace error and no editor diagnostic.
- Scope: static/non-linking only.

## 2026-08-13 — DOC-006 staged docs-sync gate

- Instrument: `node tools/src/docs-sync-gate.mjs --staged`.
- Outcome: **pass**. The gate routed daemon, scheduler, kernel-authority,
  conformance-tool and handbook changes; its nested handbook check passed
  54 × 2 documents and its generator check passed all 18 generated pages.
- Disposition: create an immutable compile/test checkpoint for required
  Ubuntu/Windows CI.

## 2026-08-13 — CI-GREEN-001 first implementation attempt

- Exact revision: `c620a630e8724b9c27705360cf8a28cf8947c426`.
- Instrument: required CI run
  [31719604296](https://github.com/agentkernel/cognitive-os/actions/runs/31719604296).
- Started/retained denominator: Ubuntu 1/1 job; Windows 1/1 job.
- Outcome: **fail** on both platforms at `cargo build --workspace --locked`.
  `error[E0433]`: `DurableExecutorStateStore` is used by
  `ProductionNativeToolExecutorRouter::open_with_artifact_store` in
  `apps/kernel-server/src/personal/tool_executor/router.rs` but is not in
  scope. Tests never started.
- Disposition: add the missing `super::DurableExecutorStateStore` import.
  No executor, registry, negative, or handbook behavior change.
- Non-claims: this compile failure is not a check, Effect, Evidence,
  verification, Task, Gate, release, or Profile result.

## 2026-08-14 — LOCAL-011 import-only compile fix

- Instrument: source review of the CI `E0433` site; linking remains `not-run`
  on `DEV-WIN-GNU-01`.
- Change: `tool_executor/router.rs` now imports
  `DurableExecutorStateStore` from the sibling `state` façade.
- Outcome: **pass** for the scoped review. Required Ubuntu/Windows CI is the
  next compile/test evidence.
- `DOCS_IMPACT_NONE="import-only compile fix; no documented behavior change"`
  because the mapped handbook pages already describe RegisteredCheckRun and
  this commit does not alter that surface.

## 2026-08-14 — NATIVE-001 focused suites at compile-fix head

- Exact revision: `c75749ccb926793480e5a8c1cea5ad606ebc2352`.
- Environment: disposable `DEV-LINUX-NATIVE-01` clone
  `/home/wuz/cos-p2t16-c75749cc` (not a shared P2-T13/T14 worktree).
- Instruments and retained outcomes:
  - `cargo test -p kernel-server registered_check -- --test-threads=1`:
    **16/16 pass** (registry, Effect reconcile, CAS/verifier and crash
    negatives, plus the independent verifier identity test).
  - `cargo test -p cognitive-kernel tool_registry -- --test-threads=1`:
    **11/11 pass**.
  - `cargo test -p kernel-server tool_executor -- --test-threads=1`:
    **76/76 pass**.
  - `cargo test -p kernel-server --test p2_t16_registered_check -- --test-threads=1`:
    **2/2 pass** (fixed C2a worker with empty env; extra argv rejected).
  - `cargo clippy -p kernel-server --all-targets -- -D warnings`: **pass**.
  - `cargo fmt --all -- --check`: **pass**.
- The first `registered_check` filter did not execute the integration-test
  binary's two tests (function names lack that substring); they were rerun
  with `--test p2_t16_registered_check` and retained.
- Non-claims: native focused pass is not Task completion, Gate, release,
  Profile, B01, or EVAL promotion. Required Windows CI remains open.

## 2026-08-14 — CI-GREEN-002 Ubuntu at compile-fix head

- Exact revision: `c75749ccb926793480e5a8c1cea5ad606ebc2352`.
- Instrument: required CI run
  [31721644351](https://github.com/agentkernel/cognitive-os/actions/runs/31721644351).
- Ubuntu `verify (ubuntu-latest)`: **pass** (workspace build/test, Clippy,
  rustfmt, codegen, consistency, handbook, conformance).
- Windows `verify (windows-latest)`: **fail**. Workspace Rust tests
  `202 passed; 4 failed`. All four failures are scheduler WorkspaceRead
  cleanup after assertions: `remove_dir_all` hits Win32 error 32 while
  `ProductionNativeToolExecutorRouter` still holds a cap-std `Dir` on
  `native-executor-state` (opened because `open()` now composes the
  RegisteredCheck ArtifactStore/state store). The four tests are
  `production_native_caller_persists_executing_before_workspace_io`,
  `private_tick_dispatches_admitted_workspace_read_through_production_router`,
  `interrupted_native_dispatch_reconciles_original_key_without_second_io`,
  and `restarted_periodic_recovery_never_repeats_an_unrecorded_workspace_read`.
- Control: every `personal::registered_check` test and the registered-check
  verifier identity test **passed** on Windows before the cleanup failures.
- Disposition: drop the router/ArtifactStore handles before `remove_dir_all`;
  no production dispatch or registry change.

## 2026-08-14 — LOCAL-012 Windows cleanup drop order

- Change: the four failing scheduler tests now drop
  `ProductionNativeToolExecutorRouter` (and any sibling ArtifactStore)
  before deleting the temporary personal layout.
- Outcome: source review **pass**. Required Windows CI must confirm the
  Win32 error 32 is gone.
- `DOCS_IMPACT_NONE="Windows test cleanup drops router Dir handles before remove_dir_all; no production or documented behavior change"`

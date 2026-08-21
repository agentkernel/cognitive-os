# 20260725 Lane-KRN Personal P0-T04 Handoff

## 1. This Session's Delivery

- Implemented a local `cognitive-store` SQLite migration adapter in
  `crates/cognitive-store/src/migration.rs`; no registry, schema, vector,
  authority-transition, or client-write contract changed.
- Added immutable migration history using
  `schema_migrations(version, digest, applied_at)`. The adapter computes a
  `sha256:<hex>` digest from migration SQL, rejects unknown recorded versions
  and content drift, and never rewrites an existing row.
- Added explicit execution modes:
  - `DryRun`: creates a non-overwriting scratch SQLite copy, then runs the plan
    only there.
  - `Apply`: creates a non-overwriting WAL-consistent pre-migration backup,
    then applies schema SQL plus metadata atomically in `BEGIN IMMEDIATE`.
- Added fail-closed checks for unsafe destinations, SQL errors, and SQLite
  `quick_check`; failed migration SQL rolls back both schema changes and
  migration metadata.
- Added focused tests covering dry-run source isolation, pre-migration backup
  contents, reapply/replay safety, SQL-derived digest drift before subsequent
  migration SQL, and rollback after invalid SQL.
- Added ADR-0017 to resolve the task card's incorrect ADR-003 reference and to
  document the authority/installation two-DB non-atomicity boundary, backup
  restore procedure, and deferred XDG realization.
- Updated `docs/plan/plan.md` task-card detail and `PROGRESS.md`; the formal Personal
  ledger records `P0-T04 done` with CI execution evidence.

## 2. Completion and Local Environment Boundary

- `P0-T04` is **done**. CI run
  [30150183941](https://github.com/agentkernel/cognitive-os/actions/runs/30150183941)
  passed on both Ubuntu and Windows runners. Its `cargo test --workspace
  --locked` step executes the new `p0_t04_migrations` integration test.
- The local Windows GNU linker still exits 121, but P0-T01 identifies it as a
  non-supported host combination. MSVC Build Tools installation was attempted
  to reproduce the supported Windows baseline locally, but Visual Studio's
  installer correctly refused because C: has only 4.07 GiB free and requires
  at least 6.8 GB. This local capacity limitation does not block the verified
  CI evidence.
- P1-T01 owns operational XDG paths, daemon lifecycle exclusion, retention,
  disk-full behavior, and coordinated two-database upgrade semantics.

## 3. Test and Evidence Status

| Check | Status | Result |
|---|---|---|
| `cargo fmt --all -- --check` | pass | Formatter check succeeded. |
| `git diff --check` | pass | No whitespace errors. |
| `cargo metadata --locked --no-deps --format-version 1` | pass | Lockfile and manifests resolve. |
| `pnpm run check:consistency` | pass | `273 requirements, 55 error codes, 63 schemas, 85 vectors`. |
| CI `cargo test --workspace --locked` | pass | CI run [30150183941](https://github.com/agentkernel/cognitive-os/actions/runs/30150183941), Ubuntu and Windows/MSVC jobs both succeeded; workspace test command includes `p0_t04_migrations`. |
| Local `cargo test -p cognitive-store --test p0_t04_migrations --locked` | not-supported | Windows GNU `x86_64-w64-mingw32-gcc` linker exits 121 before repository test execution; P0-T01 defines CI Linux and Windows/MSVC as supported baselines. |
| Personal Gates / B01-B12 / Profile | not-run | No claim created. |

No new local `artifacts/evidence/` output was generated. CI is the execution
evidence for this task; it does not create a Personal Gate, B01-B12, or
Profile claim.

## 4. Risks and Boundaries

- The migration adapter creates backups before validating recorded drift. This
  is safe (source remains untouched), but a mismatch consumes the explicit
  backup destination; callers should use fresh paths per attempt.
- `VACUUM INTO` follows a full WAL checkpoint for a consistent standalone
  SQLite copy. P1-T01 still owns XDG destination creation, permissions,
  retention, disk-full behavior, and daemon lifecycle exclusion.
- Authority and installation databases remain independent SQLite files. Do not
  claim cross-database atomic upgrade or add a generic downgrade path without
  a separately reviewed, explicitly lossless design.
- Existing authority writer/CAS/event invariants are unchanged. Do not route
  clients, Pi, or management CLI directly through this low-level API.

## 5. Next Entry

- Branch: `lane/krn-personal-p0-t04-migrations`
- Base snapshot before this working change: `87dc8ddb8099cd84fc392851232a20652356e889`
- P0-T04 has no remaining task action. The earliest blocked Phase 0 item is
  P0-T03, which requires an owner License/first-platform/distribution
  decision; do not start it without that decision.
- For a future local Windows/MSVC reproduction, free at least 2.8 GB on C:
  before installing Visual Studio C++ Build Tools, then run the focused test
  command with `cargo +1.97.1-x86_64-pc-windows-msvc`.

## 6. Snapshot

- PROGRESS updated: yes.
- Formal Personal ledger updated: yes, P0-T04 is `done`.
- Commits made by this session: `9ad5b11` (`feat(store): add fail-closed
  SQLite migration validation`); PR [#89](https://github.com/agentkernel/cognitive-os/pull/89).

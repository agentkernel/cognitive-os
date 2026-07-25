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
- Updated `plan.md` task-card detail and `PROGRESS.md`; the formal Personal
  ledger remains `P0-T04 in-progress`.

## 2. In Progress / Blocking Condition

- `P0-T04` is **not done**. Focused Rust behavior tests have not run because
  this Windows host's GNU linker exits 121 while linking dependencies. A
  gnullvm attempt also failed when build scripts could not start.
- Before changing the formal task state, run the focused test on a supported
  runner (CI Linux or Windows/MSVC):

  ```powershell
  cargo test -p cognitive-store --test p0_t04_migrations --locked
  ```

- Review the final implementation with the KRN owner before marking the task
  done. The change intentionally remains an adapter-local P0 validation
  framework, not the P1-T01 XDG migration runtime or multi-database
  coordinator.

## 3. Test and Evidence Status

| Check | Status | Result |
|---|---|---|
| `cargo fmt --all -- --check` | pass | Formatter check succeeded. |
| `git diff --check` | pass | No whitespace errors. |
| `cargo metadata --locked --no-deps --format-version 1` | pass | Lockfile and manifests resolve. |
| `pnpm run check:consistency` | pass | `273 requirements, 55 error codes, 63 schemas, 85 vectors`. |
| `cargo test -p cognitive-store --test p0_t04_migrations --locked` | blocked | Windows GNU `x86_64-w64-mingw32-gcc` linker exits 121 before repository test execution. |
| Personal Gates / B01-B12 / Profile | not-run | No claim created. |

No `artifacts/evidence/` output was generated. The source tests are not test
execution evidence until they pass on a supported runner.

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
- First action on a supported runner: run the focused test command above,
  then run the normal targeted KRN checks required by CI.
- If the focused test passes, update the P0-T04 formal ledger with the actual
  command/runner evidence, change status to `done`, update `PROGRESS.md`, and
  update draft PR #89 according to repository policy.

## 6. Snapshot

- PROGRESS updated: yes.
- Formal Personal ledger updated: yes, P0-T04 remains `in-progress`.
- Commits made by this session: `9ad5b11` (`feat(store): add fail-closed
  SQLite migration validation`); draft PR [#89](https://github.com/agentkernel/cognitive-os/pull/89).

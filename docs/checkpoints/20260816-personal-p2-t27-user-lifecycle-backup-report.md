# P2-T27 User lifecycle and backup/restore — running validation report

- Task: `P2-T27` (BR-07)
- Branch: `personal/P2-T27-user-lifecycle-backup`
- Lease: `lease/personal/P2-T27/user-lifecycle-backup`
- Claim ceiling: `hypothesis` / non-claim — no Gate, release, Profile, B01,
  EVAL, or Agent-benefit promotion
- Windows: `not-run by owner-directed Linux-only route`
- `B01-Desktop-Linux-002`: untouched (EVAL-004-only)

This is the single running report for P2-T27. Cells are appended as they
finish (`TEST-REPORT-INCREMENTAL-01`).

## D01 — public backup preflight/archive and transactional restore

### D01-IMPL-01 — archive I/O excluding secrets and authority SQLite

- Instrument: `write_personal_backup_archive` /
  `restore_personal_backup_archive` in `crates/cognitive-store/src/personal_backup.rs`.
- Outcome: authored. Archives copy config/data/state/artifact files and a
  Memory/Skill export sidecar. `authority.sqlite`, `provider-config.json`,
  bootstrap secrets, and bearer-named files are skipped. Restore preflight
  checks schema, completeness, and part digests, then overlays live files
  from a staging tree with snapshot rollback. Injected fault before apply
  leaves live state unchanged.

### D01-IMPL-02 — public CLI and management HTTP callers

- Instrument: `cognitive backup` / `restore` in `apps/admin-cli`;
  `POST /management/resource/v1/backup`, `.../backup/preflight`, and
  `.../restore` in `apps/kernel-server/src/personal/user_backup.rs`.
  Task-channel callers receive 403.
- Outcome: authored. CLI `--output` / `--archive` can run without the
  daemon. `--archive-id` talks to the management channel. Restore with
  the daemon lock present is refused on the local filesystem path; the
  in-process HTTP handler skips that check because it already holds the
  lock and still never copies SQLite.

### D01-LOCAL-01 — Windows GNU Rust tests

- Instrument: local `cargo test` on `DEV-WIN-GNU-01`.
- Outcome: `not-run by owner-directed Linux-only route` /
  `RUST-LINK-DEV-WIN-GNU-01`.

### D01-CI-01 — Ubuntu supporting CI

- Instrument: GitHub Actions `verify (ubuntu-latest)` on Draft PR
  [#226](https://github.com/agentkernel/cognitive-os/pull/226).
- Outcome at `0b8768f5`: **fail** — run
  [31937184754](https://github.com/agentkernel/cognitive-os/actions/runs/31937184754).
  `kernel-server` did not compile: `server.rs` called `user_backup::` without
  `use super::user_backup`, and `map_store_error` shadowed the local `error`
  helper (`E0433`/`E0618`). Fix follows on the same branch.

### D01-LINUX-01 — exact-revision store/CLI/HTTP cells

- Instrument: `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2` / `hal9000`, Rust 1.97.1)
  at `0b8768f5`.
- Outcome: **partial**. `cargo test -p cognitive-store --locked personal_backup`
  **22/22 pass** (P7-T02 planning tests retained; archive roundtrip, tamper,
  missing-category, schema, injected-fault, daemon-lock, and fresh-layout
  negatives included). `cargo test -p admin-cli --test p2_t27_backup_restore --locked`
  **2/2 pass**. `cargo test -p kernel-server --test p2_t27_backup_restore --locked`
  **fail** — same `E0433`/`E0618` compile errors as D01-CI-01. fmt/Clippy
  `not-run` at this revision.

### D01-CI-02 — Ubuntu supporting CI after compile/Clippy fixes

- Instrument: GitHub Actions `verify (ubuntu-latest)` on `6eea42c4` (run
  [31938000525](https://github.com/agentkernel/cognitive-os/actions/runs/31938000525)).
- Outcome: **pass** (ubuntu-latest + required-ci). Workspace build/test,
  Clippy `-D warnings`, rustfmt, handbook, consistency, and conformance
  steps succeeded. The earlier fail at `0b8768f5` (run
  [31937184754](https://github.com/agentkernel/cognitive-os/actions/runs/31937184754))
  and Clippy fail at `7d80b7a5` (run
  [31937711577](https://github.com/agentkernel/cognitive-os/actions/runs/31937711577))
  are superseded. Windows remains `not-run by owner-directed Linux-only route`. Superseded by D01-CI-03.

### D01-CI-03 — Ubuntu supporting CI after Clippy fix

- Instrument: GitHub Actions `verify (ubuntu-latest)` run
  [`31938000525`](https://github.com/agentkernel/cognitive-os/actions/runs/31938000525)
  on Draft PR [#226](https://github.com/agentkernel/cognitive-os/pull/226) at
  `6eea42c4`.
- Outcome: **pass**. `required-ci` green. Windows `not-run by owner-directed
  Linux-only route`.

### D01-LINUX-02 — exact-revision revalidation after compile fix (`7d80b7a5`)

- Instrument: `DEV-LINUX-NATIVE-01` at `7d80b7a5`.
- Outcome: **partial**. HTTP `p2_t27_backup_restore` **1/1 pass**. Clippy
  `-D warnings` **fail** on `clippy::manual_inspect` (`map_err` used only to
  clean staging on error). Fix follows (`inspect_err`).

### D01-LINUX-03 — exact-revision store/CLI/HTTP/fmt/Clippy (`6eea42c4`)

- Instrument: `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2` / `hal9000`, Rust 1.97.1)
  at `6eea42c4`.
- Outcome: **pass**. `personal_backup` **22/22**; `admin-cli` `p2_t27_backup_restore`
  **2/2**; `kernel-server` `p2_t27_backup_restore` **1/1**; `cargo fmt --all -- --check`
  **pass**; `cargo clippy --workspace --all-targets --locked -- -D warnings`
  **pass**. Windows `not-run by owner-directed Linux-only route`.
  `B01-Desktop-Linux-002` untouched.

## D02 — managed Pi install through recover

### D02-IMPL-01 — public `activate-root` / `rollback` callers

- Instrument: `admin-cli activate-root` and `admin-cli rollback` wrapping
  `activate_official_pi_root_durable` / `rollback_official_pi_root_durable`;
  current-revision test `apps/admin-cli/tests/p2_t27_pi_lifecycle.rs`.
- Outcome: authored. Reuses the P5-T01/T02/T05 stack. Process-bound upgrade/
  rollback and stale-recover epoch are fail-closed negatives. Local Rust
  `not-run` (`RUST-LINK-DEV-WIN-GNU-01`).

### D02-LINUX-01 — exact-revision `p2_t27_pi_lifecycle` (`74e3a225`)

- Instrument: `DEV-LINUX-NATIVE-01` `cargo test -p admin-cli --test p2_t27_pi_lifecycle --locked`.
- Outcome: **fail**. Rollback is a new monotonic activation (v2 + target
  binding v1 → activation_version **3**), not a rewind to version 1. Recover
  after that root move is `STATE_CONFLICT` (registration pin/digest drift).
  The public test now recovers on the original v1, then upgrades/rolls back
  the stopped root. Same revision:
  `p5_t05_identity_recover` **3/3 pass**, `p5_t05_upgrade_fencing` **4/4
  pass**, `cargo clippy -p admin-cli --all-targets --locked -- -D warnings`
  **pass**.

### D02-CI-01 — Ubuntu supporting CI (`cf7dd501`)

- Instrument: GitHub Actions `verify (ubuntu-latest)` run
  [31938624514](https://github.com/agentkernel/cognitive-os/actions/runs/31938624514)
  on Draft PR [#226](https://github.com/agentkernel/cognitive-os/pull/226).
- Outcome: **pass** (`required-ci` green). Windows `not-run by owner-directed
  Linux-only route`.

### D02-LINUX-02 — exact-revision `p2_t27_pi_lifecycle` (`cf7dd501`)

- Instrument: `DEV-LINUX-NATIVE-01` `cargo test -p admin-cli --test p2_t27_pi_lifecycle --locked`.
- Outcome: **pass** (1/1). Install → activate-root v1 → register → activate →
  process-bound upgrade/rollback negatives → pause/resume/stop → stale recover
  fail → recover (new session) → stop → upgrade v2 → monotonic rollback v3 →
  uninstall.

### D02-LOCAL-01 — Windows GNU Rust tests

- Instrument: local `cargo test` on `DEV-WIN-GNU-01`.
- Outcome: `not-run by owner-directed Linux-only route` /
  `RUST-LINK-DEV-WIN-GNU-01`.

## D03 — linux-002 equality/migration/RTO/RPO/lifecycle/cleanup matrix

### D03-IMPL-01 — destroy→restore equality, residue cleanup, bounded duration

- Instrument: `d03_destroy_restore_equality_cleanup_and_bounded_duration` in
  `crates/cognitive-store/src/personal_backup.rs`.
- Outcome: authored. Fresh-root eligible files match the archive after
  destroy→restore (RPO = archive parts). Restore duration is measured and
  bounded for the hermetic fixture; this is not an RTO SLO or Gate claim.
  Staging/snapshot trees are absent after success and after injected-fault
  rollback. Tamper, missing-part, schema, secret/SQLite, and Pi lifecycle
  cells reuse D01/D02 tests. Cross-version migration remains fail-closed
  (`SchemaIncompatible` for archive format ≠ 1).

### D03-LOCAL-01 — Windows GNU Rust tests

- Instrument: local `cargo test` on `DEV-WIN-GNU-01`.
- Outcome: `not-run by owner-directed Linux-only route` /
  `RUST-LINK-DEV-WIN-GNU-01`.

### D03-LINUX-01 — exact-revision matrix (`5a561fbf`)

- Instrument: `DEV-LINUX-NATIVE-01` (`wuz@192.168.1.2` / `hal9000`, Rust 1.97.1)
  at `5a561fbf`.
- Outcome: **pass**. `personal_backup` **23/23** (includes destroy→restore
  equality, residue cleanup, and bounded duration); `admin-cli`
  `p2_t27_backup_restore` **2/2**; `admin-cli` `p2_t27_pi_lifecycle` **1/1**;
  `kernel-server` `p2_t27_backup_restore` **1/1**;
  `cognitive-runtime` `p5_t05_identity_recover` **3/3**;
  `cognitive-runtime` `p5_t05_upgrade_fencing` **4/4**; kernel-server `--bins`
  **329/329**; workspace tests **0 failed**; `cargo fmt --all -- --check`
  **pass**; `cargo clippy --workspace --all-targets --locked -- -D warnings`
  **pass**. Residue `/tmp/cos-p2t27-*` count **0**. Windows `not-run by
  owner-directed Linux-only route`. `B01-Desktop-Linux-002` untouched.
  Cross-version migration remains fail-closed (`SchemaIncompatible`). Restore
  wall time is a hypothesis-only measurement, not an RTO SLO.

### D03-CI-01 — Ubuntu supporting CI

- Instrument: GitHub Actions `verify (ubuntu-latest)` run
  [`31939336791`](https://github.com/agentkernel/cognitive-os/actions/runs/31939336791)
  on Draft PR [#226](https://github.com/agentkernel/cognitive-os/pull/226) at
  `5a561fbf`.
- Outcome: **pass** (`required-ci` green). Windows `not-run by owner-directed
  Linux-only route`. Docs-only linux-record head `9468c3b2` Ubuntu
  [`31939577639`](https://github.com/agentkernel/cognitive-os/actions/runs/31939577639)
  also **pass**. Implementation evidence remains `5a561fbf`.

### D03-CI-02 — Ubuntu supporting CI after acceptance mapping

- Instrument: GitHub Actions `verify (ubuntu-latest)` run
  [`31939988369`](https://github.com/agentkernel/cognitive-os/actions/runs/31939988369)
  at `b296cf71`.
- Outcome: **fail**. `CURRENT_SNAPSHOT_LEASE_MISMATCH`: Layer 2 marked
  `P2-T27/D03` `done` while the active lease still named D03. Slice status
  stays `in-progress` until ready/merge. Fix follows on the same branch.

### D03-ACCEPT-01 — formal acceptance mapping

| Acceptance | Evidence |
|---|---|
| Public CLI/API backup/restore excluding secret/bearer/raw Provider/Pi/authority SQLite | D01 archive writer + `cognitive backup`/`restore` + management HTTP; `personal_backup` 23/23; CLI 2/2; HTTP 1/1 |
| Preflight + transactional restore | schema/digest/completeness preflight; injected-fault rollback; `RestorePartialRefused` |
| Fresh-root equality / RPO | destroy→restore byte-equal eligible files vs archive parts |
| Migration / tamper / missing-part | `SchemaIncompatible`, `ArchiveTampered`, incomplete category |
| Rollback / cleanup | snapshot rollback; no `restore-staging-*` / `restore-snapshot-*`; `/tmp/cos-p2t27-*` = 0 |
| RTO | finite restore wall time on the hermetic fixture; **not** an SLO/Gate claim |
| Managed Pi install→recover | D02 `p2_t27_pi_lifecycle` 1/1; `p5_t05_identity_recover` 3/3; `p5_t05_upgrade_fencing` 4/4 |
| Exact-revision linux-002 | this D03 matrix at `5a561fbf` |
| Ubuntu supporting CI | runs `31939336791` (`5a561fbf`) and `31939577639` (`9468c3b2`) pass |
| Windows | `not-run by owner-directed Linux-only route` |


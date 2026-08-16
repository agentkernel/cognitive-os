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

### D02-LOCAL-01 — Windows GNU Rust tests

- Instrument: local `cargo test` on `DEV-WIN-GNU-01`.
- Outcome: `not-run by owner-directed Linux-only route` /
  `RUST-LINK-DEV-WIN-GNU-01`.

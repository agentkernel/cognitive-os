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

- Instrument: GitHub Actions `verify (ubuntu-latest)` on the Draft PR head.
- Outcome: `not-run` (Draft PR not yet opened at this cell).

### D01-LINUX-01 — exact-revision store/CLI/HTTP cells

- Instrument: `DEV-LINUX-NATIVE-01` at a pushed revision.
- Outcome: `not-run` (waiting for an immutable pushed head).

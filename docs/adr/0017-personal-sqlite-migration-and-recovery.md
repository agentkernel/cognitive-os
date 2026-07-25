# ADR-0017: Personal SQLite Migration, Backup, and Recovery Boundary

- Status: Accepted for P0-T04 design validation
- Date: 2026-07-25
- Decision owners: CognitiveOS reference implementation maintainers
- Classification: Personal product data-layout decision. This ADR defines a
  reference-implementation persistence procedure; it is not a CognitiveOS
  specification requirement, Profile claim, or authority transition change.

## Context

ADR-0002 fixes SQLite WAL as the authoritative store, but its initial inline
schema installation did not record an immutable schema history. Personal
installation needs a failure-safe path to introduce future schema changes
without allowing an operator, client, Pi, or UI to bypass the Rust authority.

The Personal task card previously referred to "ADR-003" for this design. That
identifier is already `docs/adr/0003-json-http-sse.md`, an accepted transport
decision. This ADR is the non-conflicting migration decision and is the source
of truth for the P0-T04 design portion.

## Decision

1. Every independently managed SQLite database uses a local
   `schema_migrations(version, digest, applied_at)` table. `version` is a
   strictly increasing positive integer and `digest` is the adapter-computed
   `sha256:<hex>` checksum of the immutable migration SQL selected by the
   binary.
2. Before applying a plan, the adapter validates all existing migration rows:
   an unknown recorded version or a digest mismatch rejects the operation
   before any new migration SQL runs. The adapter never rewrites history.
3. A migration plan runs in one `BEGIN IMMEDIATE` transaction. Schema SQL and
   its metadata insert commit together; a SQL error rolls both back. The
   adapter performs `PRAGMA quick_check` before commit and fails closed if it
   does not report `ok`.
4. An apply first creates a new, caller-selected, non-overwriting SQLite backup
   via a WAL checkpoint and `VACUUM INTO`. The destination must not exist and
   must differ from the source. Failure to create this backup prevents source
   modification.
5. A dry run first creates a fresh scratch SQLite copy and applies the same
   validation and transaction only to that copy. It must not install migration
   metadata or schema changes in the source database.
6. Recovery after a failed migration is deterministic: the source remains at
   its prior committed schema because the migration transaction rolls back. If
   an operator needs to return from a successfully applied migration, they
   stop the daemon, replace the database with the preserved pre-migration
   backup under the daemon's exclusive lifecycle control, and restart the
   prior compatible binary. Automatic downgrade SQL is deliberately out of
   scope until a future migration declares and tests it.
7. The authority database and installation database are distinct SQLite files.
   P0-T04 does not invent cross-database atomicity. A future coordinated
   upgrade must preflight both copies, migrate each independently, and record
   an explicit recovery procedure before claiming a two-database upgrade.

## Data Layout

P0-T04 defined the logical roles. P1-T01 maps them to Linux XDG directories with
restrictive permissions (`PersonalDataLayout` / `prepare_personal_databases`):

| Role | Future logical location | Ownership / semantics |
|---|---|---|
| Durable authority database | `$XDG_DATA_HOME/cognitiveos/authority.sqlite` | Rust daemon only; authority state and events remain atomic per ADR-0002. |
| Durable installation database | `$XDG_DATA_HOME/cognitiveos/installations.sqlite` | Installation adapter only; not an authority-state replacement. |
| Pre-migration backup | `$XDG_STATE_HOME/cognitiveos/backups/` | Created before apply; never overwritten by the migration adapter. |
| Dry-run scratch database | `$XDG_RUNTIME_DIR/cognitiveos/migration/` | Ephemeral, daemon-private; dry run must not mutate durable data. |

Directory creation, Unix 0700/0600 modes, exclusive `migration.lock`, and
dual-database prepare are implemented in-store. Long-term backup retention
policy, full daemon single-instance lifecycle, and coordinated two-database
atomic upgrade remain later Personal work. This is not a release or Profile
claim.

## Consequences

- The adapter API is local to `cognitive-store`; it introduces no registry row,
  schema, transition, vector, wire DTO, or public client-write capability.
- Successful migration means only that the local SQLite transaction committed.
  It does not assert a Profile, Gate, release readiness, or external operation
  success.
- Callers must supply destination paths. This prevents implicit path selection,
  avoids secret-bearing configuration, and makes backup retention an explicit
  P1 operational policy.

## Alternatives Considered

### `PRAGMA user_version` without a digest ledger

Rejected: it cannot detect a changed migration body for an already-applied
version and therefore cannot fail closed on plan drift.

### In-place dry run with rollback

Rejected: although transactional DDL can roll back, a scratch-copy run proves
the intended isolation boundary and avoids exposing a live authority database
to validation behavior.

### Automatic downgrade migrations

Rejected for P0: a generic downgrade can silently discard data or diverge
across the authority and installation databases. Restore from the preserved
backup is the only defined rollback until a future migration explicitly proves
lossless downgrade behavior.

## Validation Status

P0-T04 adds focused tests for scratch dry run, apply/replay, digest mismatch
with zero subsequent SQL side effect, and transactional failure recovery.
They remain subject to execution on a supported Rust toolchain. This ADR is a
design and local-adapter decision only; it does not complete P1-T01 or G0.

P1-T01 realizes the XDG path roles above in `cognitive-store` (`layout` +
`personal_db`): directory creation with Unix 0700, database files 0600,
exclusive `migration.lock`, and dual-database prepare that applies the
versioned plans independently with non-overwriting backups under
`$XDG_STATE_HOME/cognitiveos/backups/`. Cross-database atomic upgrade remains
out of scope. P1-T01 does not claim G0, B01-B12, or Profile conformance.

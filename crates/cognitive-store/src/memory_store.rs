//! Durable append-only Memory admission storage.

use crate::migration::MigrationPlanEntry;

/// Migration v16: proposal, decision, and admitted-object records for the
/// daemon-private Memory admission path. Retrieval indexes and lifecycle
/// transitions intentionally belong to later P4 tasks.
pub const MEMORY_ADMISSION_SCHEMA_V16: &str = "
CREATE TABLE IF NOT EXISTS memory_candidates (
  candidate_id TEXT PRIMARY KEY,
  source_id TEXT NOT NULL REFERENCES workspace_context_sources(source_id),
  source_digest TEXT NOT NULL,
  source_provenance_ref TEXT NOT NULL,
  governance_scope TEXT NOT NULL,
  target_scope TEXT NOT NULL,
  purpose TEXT NOT NULL CHECK (length(trim(purpose)) > 0),
  retention_expires_at_unix_seconds INTEGER NOT NULL,
  observed_at_unix_seconds INTEGER NOT NULL,
  canonical_json TEXT NOT NULL
) STRICT;
CREATE TRIGGER IF NOT EXISTS memory_candidates_append_only_update
BEFORE UPDATE ON memory_candidates
BEGIN SELECT RAISE(ABORT, 'append-only: memory candidate is immutable'); END;
CREATE TRIGGER IF NOT EXISTS memory_candidates_append_only_delete
BEFORE DELETE ON memory_candidates
BEGIN SELECT RAISE(ABORT, 'append-only: memory candidate is immutable'); END;

CREATE TABLE IF NOT EXISTS memory_admission_decisions (
  decision_id TEXT PRIMARY KEY,
  candidate_id TEXT NOT NULL UNIQUE REFERENCES memory_candidates(candidate_id),
  candidate_digest TEXT NOT NULL,
  decision TEXT NOT NULL CHECK (decision IN ('admit', 'reject', 'review', 'quarantine')),
  policy_version INTEGER NOT NULL CHECK (policy_version >= 1),
  reason_codes_json TEXT NOT NULL,
  canonical_json TEXT NOT NULL
) STRICT;
CREATE TRIGGER IF NOT EXISTS memory_admission_decisions_append_only_update
BEFORE UPDATE ON memory_admission_decisions
BEGIN SELECT RAISE(ABORT, 'append-only: memory admission decision is immutable'); END;
CREATE TRIGGER IF NOT EXISTS memory_admission_decisions_append_only_delete
BEFORE DELETE ON memory_admission_decisions
BEGIN SELECT RAISE(ABORT, 'append-only: memory admission decision is immutable'); END;

CREATE TABLE IF NOT EXISTS memory_objects (
  memory_id TEXT PRIMARY KEY,
  candidate_id TEXT NOT NULL UNIQUE REFERENCES memory_candidates(candidate_id),
  decision_id TEXT NOT NULL UNIQUE REFERENCES memory_admission_decisions(decision_id),
  canonical_json TEXT NOT NULL
) STRICT;
CREATE TRIGGER IF NOT EXISTS memory_objects_append_only_update
BEFORE UPDATE ON memory_objects
BEGIN SELECT RAISE(ABORT, 'append-only: memory object is immutable'); END;
CREATE TRIGGER IF NOT EXISTS memory_objects_append_only_delete
BEFORE DELETE ON memory_objects
BEGIN SELECT RAISE(ABORT, 'append-only: memory object is immutable'); END;
";

/// Version-16 Memory admission migration entry.
pub fn memory_admission_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(16, MEMORY_ADMISSION_SCHEMA_V16)
}

/// Migration v17: a disposable SQLite FTS5 index for daemon-private Memory
/// discovery. Authoritative Memory, admission, and source rows remain the
/// query filter and rebuild source of truth.
pub const MEMORY_SEARCH_SCHEMA_V17: &str = "
CREATE VIRTUAL TABLE IF NOT EXISTS memory_search_fts USING fts5(
  memory_id UNINDEXED,
  source_text,
  tokenize='unicode61'
);
";

/// Version-17 Memory FTS retrieval migration entry.
pub fn memory_search_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(17, MEMORY_SEARCH_SCHEMA_V17)
}

/// Migration v18: append-only lifecycle audit facts for daemon-owned Memory
/// forget operations. The unique Memory identity makes forget idempotency
/// explicit while retaining the original admission rows for audit.
pub const MEMORY_LIFECYCLE_SCHEMA_V18: &str = "
CREATE TABLE IF NOT EXISTS memory_tombstones (
  lifecycle_id TEXT PRIMARY KEY,
  memory_id TEXT NOT NULL UNIQUE REFERENCES memory_objects(memory_id),
  action TEXT NOT NULL CHECK (action = 'forget'),
  occurred_at_unix_seconds INTEGER NOT NULL,
  reason TEXT NOT NULL CHECK (length(trim(reason)) > 0),
  canonical_json TEXT NOT NULL
) STRICT;
CREATE TRIGGER IF NOT EXISTS memory_tombstones_append_only_update
BEFORE UPDATE ON memory_tombstones
BEGIN SELECT RAISE(ABORT, 'append-only: Memory tombstone is immutable'); END;
CREATE TRIGGER IF NOT EXISTS memory_tombstones_append_only_delete
BEFORE DELETE ON memory_tombstones
BEGIN SELECT RAISE(ABORT, 'append-only: Memory tombstone is immutable'); END;
";

/// Version-18 Memory lifecycle migration entry.
pub fn memory_lifecycle_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(18, MEMORY_LIFECYCLE_SCHEMA_V18)
}

/// Migration v19: extend the immutable lifecycle fact projection with the
/// daemon-owned retention-expiry action without rewriting migration v18.
pub const MEMORY_EXPIRY_SCHEMA_V19: &str = "
DROP TRIGGER IF EXISTS memory_tombstones_append_only_update;
DROP TRIGGER IF EXISTS memory_tombstones_append_only_delete;
ALTER TABLE memory_tombstones RENAME TO memory_tombstones_v18;
CREATE TABLE memory_tombstones (
  lifecycle_id TEXT PRIMARY KEY,
  memory_id TEXT NOT NULL UNIQUE REFERENCES memory_objects(memory_id),
  action TEXT NOT NULL CHECK (action IN ('forget', 'expire')),
  occurred_at_unix_seconds INTEGER NOT NULL,
  reason TEXT NOT NULL CHECK (length(trim(reason)) > 0),
  canonical_json TEXT NOT NULL
) STRICT;
INSERT INTO memory_tombstones SELECT * FROM memory_tombstones_v18;
DROP TABLE memory_tombstones_v18;
CREATE TRIGGER memory_tombstones_append_only_update
BEFORE UPDATE ON memory_tombstones
BEGIN SELECT RAISE(ABORT, 'append-only: Memory lifecycle fact is immutable'); END;
CREATE TRIGGER memory_tombstones_append_only_delete
BEFORE DELETE ON memory_tombstones
BEGIN SELECT RAISE(ABORT, 'append-only: Memory lifecycle fact is immutable'); END;
";

/// Version-19 Memory expiry lifecycle migration entry.
pub fn memory_expiry_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(19, MEMORY_EXPIRY_SCHEMA_V19)
}

/// Migration v20: preserve the version/CAS lineage of replacement Memory
/// objects while allowing the v19 lifecycle projection to record supersede.
pub const MEMORY_VERSION_SCHEMA_V20: &str = "
DROP TRIGGER IF EXISTS memory_tombstones_append_only_update;
DROP TRIGGER IF EXISTS memory_tombstones_append_only_delete;
ALTER TABLE memory_tombstones RENAME TO memory_tombstones_v19;
CREATE TABLE memory_tombstones (
  lifecycle_id TEXT PRIMARY KEY,
  memory_id TEXT NOT NULL UNIQUE REFERENCES memory_objects(memory_id),
  action TEXT NOT NULL CHECK (action IN ('forget', 'expire', 'supersede')),
  occurred_at_unix_seconds INTEGER NOT NULL,
  reason TEXT NOT NULL CHECK (length(trim(reason)) > 0),
  canonical_json TEXT NOT NULL
) STRICT;
INSERT INTO memory_tombstones SELECT * FROM memory_tombstones_v19;
DROP TABLE memory_tombstones_v19;
CREATE TRIGGER memory_tombstones_append_only_update
BEFORE UPDATE ON memory_tombstones
BEGIN SELECT RAISE(ABORT, 'append-only: Memory lifecycle fact is immutable'); END;
CREATE TRIGGER memory_tombstones_append_only_delete
BEFORE DELETE ON memory_tombstones
BEGIN SELECT RAISE(ABORT, 'append-only: Memory lifecycle fact is immutable'); END;
CREATE TABLE IF NOT EXISTS memory_object_versions (
  memory_id TEXT PRIMARY KEY REFERENCES memory_objects(memory_id),
  version INTEGER NOT NULL CHECK (version >= 1),
  supersedes_memory_id TEXT REFERENCES memory_objects(memory_id),
  UNIQUE (supersedes_memory_id)
) STRICT;
CREATE TRIGGER IF NOT EXISTS memory_object_versions_append_only_update
BEFORE UPDATE ON memory_object_versions
BEGIN SELECT RAISE(ABORT, 'append-only: Memory version lineage is immutable'); END;
CREATE TRIGGER IF NOT EXISTS memory_object_versions_append_only_delete
BEFORE DELETE ON memory_object_versions
BEGIN SELECT RAISE(ABORT, 'append-only: Memory version lineage is immutable'); END;
INSERT INTO memory_object_versions (memory_id, version)
SELECT memory_id, 1 FROM memory_objects
WHERE NOT EXISTS (SELECT 1 FROM memory_object_versions WHERE memory_id = memory_objects.memory_id);
";

/// Version-20 Memory version lineage migration entry.
pub fn memory_version_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(20, MEMORY_VERSION_SCHEMA_V20)
}

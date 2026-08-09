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

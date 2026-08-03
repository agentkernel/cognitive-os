//! Private persistence layout for daemon-only worker authorization inputs.
//!
//! Candidate proposals are durable, append-only observations. Their presence
//! never authorizes dispatch; daemon admission will later validate them before
//! creating an Intent, Effect, WorkerIterationAuthorization, or debit.

use crate::migration::MigrationPlanEntry;

/// Migration v4: immutable operation candidate proposal persistence.
pub const WORKER_AUTHORIZATION_SCHEMA_V4: &str = "
CREATE TABLE IF NOT EXISTS operation_candidate_proposals (
  candidate_id              TEXT PRIMARY KEY,
  task_ref                  TEXT NOT NULL,
  contract_epoch            INTEGER NOT NULL CHECK (contract_epoch >= 1),
  candidate_source_ref      TEXT NOT NULL,
  tool_ref                  TEXT NOT NULL,
  action                    TEXT NOT NULL,
  target                    TEXT NOT NULL,
  parameters_digest         TEXT NOT NULL,
  expected_state_version    INTEGER NOT NULL CHECK (expected_state_version >= 1),
  operation_descriptor_ref  TEXT NOT NULL,
  canonical_json            TEXT NOT NULL
) STRICT;

CREATE TRIGGER IF NOT EXISTS operation_candidate_proposals_append_only_update
BEFORE UPDATE ON operation_candidate_proposals
BEGIN SELECT RAISE(ABORT, 'append-only: candidate proposals are immutable'); END;

CREATE TRIGGER IF NOT EXISTS operation_candidate_proposals_append_only_delete
BEFORE DELETE ON operation_candidate_proposals
BEGIN SELECT RAISE(ABORT, 'append-only: candidate proposals are immutable'); END;
";

/// The version-4 candidate persistence migration entry.
pub fn worker_authorization_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(4, WORKER_AUTHORIZATION_SCHEMA_V4)
}

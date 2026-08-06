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

/// Migration v5: daemon-only immutable operation descriptor registry.
pub const DAEMON_OPERATION_DESCRIPTOR_SCHEMA_V5: &str = "
CREATE TABLE IF NOT EXISTS daemon_operation_descriptors (
  descriptor_id       TEXT PRIMARY KEY,
  operation_id        TEXT NOT NULL,
  action              TEXT NOT NULL,
  effect_class        TEXT NOT NULL CHECK (effect_class IN ('pure', 'local_ephemeral', 'governed_external', 'emergency_safety')),
  executor            TEXT NOT NULL,
  queryable           INTEGER NOT NULL CHECK (queryable IN (0, 1)),
  idempotent          INTEGER NOT NULL CHECK (idempotent IN (0, 1)),
  descriptor_version  INTEGER NOT NULL CHECK (descriptor_version >= 1),
  canonical_json      TEXT NOT NULL
) STRICT;

CREATE TRIGGER IF NOT EXISTS daemon_operation_descriptors_append_only_update
BEFORE UPDATE ON daemon_operation_descriptors
BEGIN SELECT RAISE(ABORT, 'append-only: daemon operation descriptors are immutable'); END;

CREATE TRIGGER IF NOT EXISTS daemon_operation_descriptors_append_only_delete
BEFORE DELETE ON daemon_operation_descriptors
BEGIN SELECT RAISE(ABORT, 'append-only: daemon operation descriptors are immutable'); END;
";

/// The version-5 daemon descriptor registry migration entry.
pub fn daemon_operation_descriptor_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(5, DAEMON_OPERATION_DESCRIPTOR_SCHEMA_V5)
}

/// Migration v6: daemon-only append-only authorization decision snapshots.
pub const DAEMON_AUTHORIZATION_SNAPSHOT_SCHEMA_V6: &str = "
CREATE TABLE IF NOT EXISTS daemon_authorization_snapshots (
  snapshot_sequence       INTEGER PRIMARY KEY AUTOINCREMENT,
  snapshot_id             TEXT NOT NULL UNIQUE,
  subject_ref             TEXT NOT NULL,
  target_ref              TEXT NOT NULL,
  action                  TEXT NOT NULL,
  purpose                 TEXT NOT NULL,
  grant_epoch             INTEGER NOT NULL CHECK (grant_epoch >= 1),
  capability_set_version  INTEGER NOT NULL CHECK (capability_set_version >= 1),
  revocation_epoch         INTEGER NOT NULL CHECK (revocation_epoch >= 1),
  observed_at             TEXT NOT NULL,
  canonical_json          TEXT NOT NULL
) STRICT;
CREATE INDEX IF NOT EXISTS daemon_authorization_snapshots_binding
ON daemon_authorization_snapshots (subject_ref, target_ref, action, purpose, snapshot_sequence DESC);
CREATE TRIGGER IF NOT EXISTS daemon_authorization_snapshots_append_only_update
BEFORE UPDATE ON daemon_authorization_snapshots
BEGIN SELECT RAISE(ABORT, 'append-only: daemon authorization snapshots are immutable'); END;
CREATE TRIGGER IF NOT EXISTS daemon_authorization_snapshots_append_only_delete
BEFORE DELETE ON daemon_authorization_snapshots
BEGIN SELECT RAISE(ABORT, 'append-only: daemon authorization snapshots are immutable'); END;
";

/// The version-6 daemon authorization snapshot migration entry.
pub fn daemon_authorization_snapshot_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(6, DAEMON_AUTHORIZATION_SNAPSHOT_SCHEMA_V6)
}

/// Migration v7: immutable pre-dispatch WorkerIterationAuthorization rows.
pub const WORKER_ITERATION_AUTHORIZATION_SCHEMA_V7: &str = "
CREATE TABLE IF NOT EXISTS worker_iteration_authorizations (
  authorization_id             TEXT PRIMARY KEY,
  worker_authorization_root_id TEXT NOT NULL,
  task_ref                     TEXT NOT NULL,
  contract_epoch               INTEGER NOT NULL CHECK (contract_epoch >= 1),
  loop_object_id               TEXT NOT NULL,
  iteration                    INTEGER NOT NULL CHECK (iteration >= 1),
  expected_loop_version        INTEGER NOT NULL CHECK (expected_loop_version >= 1),
  selected_candidate_id        TEXT NOT NULL UNIQUE,
  intent_id                    TEXT NOT NULL UNIQUE,
  effect_object_id             TEXT NOT NULL UNIQUE,
  budget_id                    TEXT NOT NULL,
  budget_charge_json           TEXT NOT NULL,
  action_fingerprint           TEXT NOT NULL,
  issued_fencing_epoch         INTEGER NOT NULL CHECK (issued_fencing_epoch >= 1),
  canonical_json               TEXT NOT NULL,
  UNIQUE (worker_authorization_root_id, iteration),
  UNIQUE (loop_object_id, iteration)
) STRICT;
CREATE TRIGGER IF NOT EXISTS worker_iteration_authorizations_append_only_update
BEFORE UPDATE ON worker_iteration_authorizations
BEGIN SELECT RAISE(ABORT, 'append-only: worker iteration authorizations are immutable'); END;
CREATE TRIGGER IF NOT EXISTS worker_iteration_authorizations_append_only_delete
BEFORE DELETE ON worker_iteration_authorizations
BEGIN SELECT RAISE(ABORT, 'append-only: worker iteration authorizations are immutable'); END;
";

/// The version-7 worker authorization storage migration entry.
pub fn worker_iteration_authorization_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(7, WORKER_ITERATION_AUTHORIZATION_SCHEMA_V7)
}

/// Migration v8: append-only one-time WIA consumption records.
pub const WORKER_ITERATION_AUTHORIZATION_CONSUMPTION_SCHEMA_V8: &str = "
CREATE TABLE IF NOT EXISTS worker_iteration_authorization_consumptions (
  authorization_id       TEXT PRIMARY KEY REFERENCES worker_iteration_authorizations(authorization_id),
  worker_attempt_id      TEXT NOT NULL UNIQUE,
  consumed_fencing_epoch INTEGER NOT NULL CHECK (consumed_fencing_epoch >= 1),
  consumed_at            TEXT NOT NULL,
  canonical_json         TEXT NOT NULL
) STRICT;
CREATE TRIGGER IF NOT EXISTS worker_iteration_authorization_consumptions_append_only_update
BEFORE UPDATE ON worker_iteration_authorization_consumptions
BEGIN SELECT RAISE(ABORT, 'append-only: worker authorization consumption is immutable'); END;
CREATE TRIGGER IF NOT EXISTS worker_iteration_authorization_consumptions_append_only_delete
BEFORE DELETE ON worker_iteration_authorization_consumptions
BEGIN SELECT RAISE(ABORT, 'append-only: worker authorization consumption is immutable'); END;
";

/// The version-8 WIA consumption migration entry.
pub fn worker_iteration_authorization_consumption_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(8, WORKER_ITERATION_AUTHORIZATION_CONSUMPTION_SCHEMA_V8)
}

/// Migration v9: daemon-private exact scheduler lease binding for a WIA
/// handoff. A separate append-only table preserves older unbound consumption
/// evidence without pretending it is safe to reconcile or release.
pub const WORKER_AUTHORIZATION_LEASE_BINDING_SCHEMA_V9: &str = "
CREATE TABLE IF NOT EXISTS worker_authorization_scheduler_lease_bindings (
  authorization_id       TEXT PRIMARY KEY REFERENCES worker_iteration_authorization_consumptions(authorization_id),
  task_ref               TEXT NOT NULL,
  contract_epoch         INTEGER NOT NULL CHECK (contract_epoch >= 1),
  lease_owner            TEXT NOT NULL CHECK (lease_owner <> ''),
  lease_epoch            INTEGER NOT NULL CHECK (lease_epoch >= 1)
) STRICT;
CREATE TRIGGER IF NOT EXISTS worker_authorization_scheduler_lease_bindings_append_only_update
BEFORE UPDATE ON worker_authorization_scheduler_lease_bindings
BEGIN SELECT RAISE(ABORT, 'append-only: worker authorization lease binding is immutable'); END;
CREATE TRIGGER IF NOT EXISTS worker_authorization_scheduler_lease_bindings_append_only_delete
BEFORE DELETE ON worker_authorization_scheduler_lease_bindings
BEGIN SELECT RAISE(ABORT, 'append-only: worker authorization lease binding is immutable'); END;
";

/// The version-9 WIA-to-scheduler lease binding migration entry.
pub fn worker_authorization_lease_binding_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(9, WORKER_AUTHORIZATION_LEASE_BINDING_SCHEMA_V9)
}

/// Migration v10: daemon-private verified-continuation evidence and authority.
/// These records intentionally do not alter the public WorkerIterationAuthorization
/// schema: candidate WIA remains pre-dispatch `DECIDE -> ACT` authority.
pub const CONTINUATION_AUTHORITY_SCHEMA_V10: &str = "
CREATE TABLE IF NOT EXISTS fixed_post_states (
  fixed_post_state_id    TEXT PRIMARY KEY,
  task_ref               TEXT NOT NULL,
  contract_epoch         INTEGER NOT NULL CHECK (contract_epoch >= 1),
  loop_object_id         TEXT NOT NULL,
  subject_domain         TEXT NOT NULL,
  subject_object_id      TEXT NOT NULL,
  subject_version        INTEGER NOT NULL CHECK (subject_version >= 1),
  recorded_fencing_epoch INTEGER NOT NULL CHECK (recorded_fencing_epoch >= 1),
  canonical_json         TEXT NOT NULL
) STRICT;
CREATE TRIGGER IF NOT EXISTS fixed_post_states_append_only_update
BEFORE UPDATE ON fixed_post_states
BEGIN SELECT RAISE(ABORT, 'append-only: fixed post-state is immutable'); END;
CREATE TRIGGER IF NOT EXISTS fixed_post_states_append_only_delete
BEFORE DELETE ON fixed_post_states
BEGIN SELECT RAISE(ABORT, 'append-only: fixed post-state is immutable'); END;

CREATE TABLE IF NOT EXISTS verification_requests (
  verification_request_id TEXT PRIMARY KEY,
  fixed_post_state_id     TEXT NOT NULL REFERENCES fixed_post_states(fixed_post_state_id),
  task_ref                TEXT NOT NULL,
  contract_epoch          INTEGER NOT NULL CHECK (contract_epoch >= 1),
  loop_object_id          TEXT NOT NULL,
  expected_loop_version   INTEGER NOT NULL CHECK (expected_loop_version >= 1),
  verifier_ref            TEXT NOT NULL,
  verifier_version        TEXT NOT NULL,
  criteria_json           TEXT NOT NULL,
  issued_fencing_epoch    INTEGER NOT NULL CHECK (issued_fencing_epoch >= 1),
  canonical_json          TEXT NOT NULL
) STRICT;
CREATE TRIGGER IF NOT EXISTS verification_requests_append_only_update
BEFORE UPDATE ON verification_requests
BEGIN SELECT RAISE(ABORT, 'append-only: verification request is immutable'); END;
CREATE TRIGGER IF NOT EXISTS verification_requests_append_only_delete
BEFORE DELETE ON verification_requests
BEGIN SELECT RAISE(ABORT, 'append-only: verification request is immutable'); END;

CREATE TABLE IF NOT EXISTS verification_reports (
  verification_report_id  TEXT PRIMARY KEY,
  verification_request_id TEXT NOT NULL REFERENCES verification_requests(verification_request_id),
  fixed_post_state_id     TEXT NOT NULL REFERENCES fixed_post_states(fixed_post_state_id),
  verifier_ref            TEXT NOT NULL,
  verifier_version        TEXT NOT NULL,
  status                  TEXT NOT NULL CHECK (status IN ('passed', 'failed', 'indeterminate')),
  evidence_refs_json      TEXT NOT NULL,
  completed_at            TEXT NOT NULL,
  recorded_fencing_epoch  INTEGER NOT NULL CHECK (recorded_fencing_epoch >= 1),
  canonical_json          TEXT NOT NULL,
  UNIQUE (verification_request_id, verifier_ref, verifier_version)
) STRICT;
CREATE TRIGGER IF NOT EXISTS verification_reports_append_only_update
BEFORE UPDATE ON verification_reports
BEGIN SELECT RAISE(ABORT, 'append-only: verification report is immutable'); END;
CREATE TRIGGER IF NOT EXISTS verification_reports_append_only_delete
BEFORE DELETE ON verification_reports
BEGIN SELECT RAISE(ABORT, 'append-only: verification report is immutable'); END;

CREATE TABLE IF NOT EXISTS continuation_authorizations (
  continuation_authorization_id TEXT PRIMARY KEY,
  task_ref                      TEXT NOT NULL,
  contract_epoch                INTEGER NOT NULL CHECK (contract_epoch >= 1),
  loop_object_id                TEXT NOT NULL,
  iteration                     INTEGER NOT NULL CHECK (iteration >= 1),
  expected_loop_version         INTEGER NOT NULL CHECK (expected_loop_version >= 1),
  checkpoint_id                 TEXT NOT NULL,
  budget_id                     TEXT NOT NULL,
  budget_charge_json            TEXT NOT NULL,
  verification_report_id        TEXT NOT NULL REFERENCES verification_reports(verification_report_id),
  issued_fencing_epoch          INTEGER NOT NULL CHECK (issued_fencing_epoch >= 1),
  canonical_json                TEXT NOT NULL,
  UNIQUE (loop_object_id, iteration)
) STRICT;
CREATE TABLE IF NOT EXISTS continuation_authorization_consumptions (
  continuation_authorization_id TEXT PRIMARY KEY REFERENCES continuation_authorizations(continuation_authorization_id),
  consumed_fencing_epoch        INTEGER NOT NULL CHECK (consumed_fencing_epoch >= 1),
  consumed_at                   TEXT NOT NULL,
  canonical_json                TEXT NOT NULL
) STRICT;
CREATE TRIGGER IF NOT EXISTS continuation_authorizations_append_only_update
BEFORE UPDATE ON continuation_authorizations
BEGIN SELECT RAISE(ABORT, 'append-only: continuation authorization is immutable'); END;
CREATE TRIGGER IF NOT EXISTS continuation_authorizations_append_only_delete
BEFORE DELETE ON continuation_authorizations
BEGIN SELECT RAISE(ABORT, 'append-only: continuation authorization is immutable'); END;
CREATE TRIGGER IF NOT EXISTS continuation_authorization_consumptions_append_only_update
BEFORE UPDATE ON continuation_authorization_consumptions
BEGIN SELECT RAISE(ABORT, 'append-only: continuation consumption is immutable'); END;
CREATE TRIGGER IF NOT EXISTS continuation_authorization_consumptions_append_only_delete
BEFORE DELETE ON continuation_authorization_consumptions
BEGIN SELECT RAISE(ABORT, 'append-only: continuation consumption is immutable'); END;
";

/// The version-10 private verified-continuation persistence entry.
pub fn continuation_authority_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(10, CONTINUATION_AUTHORITY_SCHEMA_V10)
}

/// Private continuation handoff bindings; kept separate from the v10 issue
/// records so a failed harness entry remains recoverable without mutating it.
pub const CONTINUATION_AUTHORITY_CONSUMPTION_SCHEMA_V11: &str = "
CREATE TABLE IF NOT EXISTS continuation_authorization_scheduler_lease_bindings (
  continuation_authorization_id TEXT PRIMARY KEY REFERENCES continuation_authorizations(continuation_authorization_id),
  task_ref                      TEXT NOT NULL,
  contract_epoch                INTEGER NOT NULL CHECK (contract_epoch >= 1),
  lease_owner                   TEXT NOT NULL,
  lease_epoch                   INTEGER NOT NULL CHECK (lease_epoch >= 1)
) STRICT;
CREATE TRIGGER IF NOT EXISTS continuation_authorization_scheduler_lease_bindings_append_only_update
BEFORE UPDATE ON continuation_authorization_scheduler_lease_bindings
BEGIN SELECT RAISE(ABORT, 'append-only: continuation scheduler lease binding is immutable'); END;
CREATE TRIGGER IF NOT EXISTS continuation_authorization_scheduler_lease_bindings_append_only_delete
BEFORE DELETE ON continuation_authorization_scheduler_lease_bindings
BEGIN SELECT RAISE(ABORT, 'append-only: continuation scheduler lease binding is immutable'); END;
";

/// The version-11 private continuation handoff persistence entry.
pub fn continuation_authority_consumption_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(11, CONTINUATION_AUTHORITY_CONSUMPTION_SCHEMA_V11)
}

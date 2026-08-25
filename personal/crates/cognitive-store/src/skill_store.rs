//! Durable append-only storage for daemon-private local Skill imports.

use crate::migration::MigrationPlanEntry;

/// Migration v21: immutable local Skill package, revision, and eligibility
/// binding records. Package bytes and scripts stay outside this authority
/// schema; recorded digests and provenance identify the reviewed content.
pub const SKILL_PACKAGE_SCHEMA_V21: &str = "
CREATE TABLE skill_packages (
  package_id TEXT PRIMARY KEY,
  workspace_scope TEXT NOT NULL CHECK (length(trim(workspace_scope)) > 0),
  local_source_path TEXT NOT NULL CHECK (length(trim(local_source_path)) > 0),
  provenance_ref TEXT NOT NULL CHECK (length(trim(provenance_ref)) > 0),
  manifest_digest TEXT NOT NULL CHECK (length(trim(manifest_digest)) > 0),
  canonical_json TEXT NOT NULL
) STRICT;
CREATE TRIGGER skill_packages_append_only_update
BEFORE UPDATE ON skill_packages
BEGIN SELECT RAISE(ABORT, 'append-only: Skill package is immutable'); END;
CREATE TRIGGER skill_packages_append_only_delete
BEFORE DELETE ON skill_packages
BEGIN SELECT RAISE(ABORT, 'append-only: Skill package is immutable'); END;

CREATE TABLE skill_revisions (
  revision_id TEXT PRIMARY KEY,
  package_id TEXT NOT NULL REFERENCES skill_packages(package_id),
  content_digest TEXT NOT NULL UNIQUE CHECK (length(trim(content_digest)) > 0),
  compatibility TEXT NOT NULL CHECK (compatibility IN ('compatible', 'incompatible')),
  canonical_json TEXT NOT NULL
) STRICT;
CREATE TRIGGER skill_revisions_append_only_update
BEFORE UPDATE ON skill_revisions
BEGIN SELECT RAISE(ABORT, 'append-only: Skill revision is immutable'); END;
CREATE TRIGGER skill_revisions_append_only_delete
BEFORE DELETE ON skill_revisions
BEGIN SELECT RAISE(ABORT, 'append-only: Skill revision is immutable'); END;

CREATE TABLE skill_bindings (
  binding_id TEXT PRIMARY KEY,
  revision_id TEXT NOT NULL REFERENCES skill_revisions(revision_id),
  workspace_scope TEXT NOT NULL CHECK (length(trim(workspace_scope)) > 0),
  target_kind TEXT NOT NULL CHECK (target_kind IN ('agent', 'task', 'workspace')),
  target_ref TEXT NOT NULL CHECK (length(trim(target_ref)) > 0),
  status TEXT NOT NULL CHECK (status IN ('active', 'revoked')),
  canonical_json TEXT NOT NULL,
  UNIQUE (revision_id, workspace_scope, target_kind, target_ref)
) STRICT;
CREATE TRIGGER skill_bindings_append_only_update
BEFORE UPDATE ON skill_bindings
BEGIN SELECT RAISE(ABORT, 'append-only: Skill binding is immutable'); END;
CREATE TRIGGER skill_bindings_append_only_delete
BEFORE DELETE ON skill_bindings
BEGIN SELECT RAISE(ABORT, 'append-only: Skill binding is immutable'); END;
";

/// Version-21 Skill package migration entry.
pub fn skill_package_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(21, SKILL_PACKAGE_SCHEMA_V21)
}

/// Migration v22: binding revocations are separate immutable lifecycle facts
/// so the original scope, target, and revision evidence remain explainable.
pub const SKILL_BINDING_REVOCATION_SCHEMA_V22: &str = "
CREATE TABLE skill_binding_revocations (
  revocation_id TEXT PRIMARY KEY,
  binding_id TEXT NOT NULL UNIQUE REFERENCES skill_bindings(binding_id),
  reason TEXT NOT NULL CHECK (length(trim(reason)) > 0),
  canonical_json TEXT NOT NULL
) STRICT;
CREATE TRIGGER skill_binding_revocations_append_only_update
BEFORE UPDATE ON skill_binding_revocations
BEGIN SELECT RAISE(ABORT, 'append-only: Skill binding revocation is immutable'); END;
CREATE TRIGGER skill_binding_revocations_append_only_delete
BEFORE DELETE ON skill_binding_revocations
BEGIN SELECT RAISE(ABORT, 'append-only: Skill binding revocation is immutable'); END;
";

/// Version-22 Skill binding revocation migration entry.
pub fn skill_binding_revocation_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(22, SKILL_BINDING_REVOCATION_SCHEMA_V22)
}

/// Migration v23: one immutable replacement lineage per prior Skill revision.
/// Existing bindings remain exact pins to their recorded revision instead of
/// silently drifting to a newer package revision.
pub const SKILL_REVISION_LINEAGE_SCHEMA_V23: &str = "
CREATE TABLE skill_revision_lineage (
  revision_id TEXT PRIMARY KEY REFERENCES skill_revisions(revision_id),
  supersedes_revision_id TEXT NOT NULL UNIQUE REFERENCES skill_revisions(revision_id),
  canonical_json TEXT NOT NULL
) STRICT;
CREATE TRIGGER skill_revision_lineage_append_only_update
BEFORE UPDATE ON skill_revision_lineage
BEGIN SELECT RAISE(ABORT, 'append-only: Skill revision lineage is immutable'); END;
CREATE TRIGGER skill_revision_lineage_append_only_delete
BEFORE DELETE ON skill_revision_lineage
BEGIN SELECT RAISE(ABORT, 'append-only: Skill revision lineage is immutable'); END;
";

/// Version-23 Skill revision lineage migration entry.
pub fn skill_revision_lineage_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(23, SKILL_REVISION_LINEAGE_SCHEMA_V23)
}

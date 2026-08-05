//! Durable append-only Context request and view persistence.
//!
//! A TaskContract pins an immutable ContextRequest. Each ContextView is a
//! separate immutable resolution artifact bound to that request; neither row
//! is a lifecycle transition or a grant of execution authority.

use crate::migration::MigrationPlanEntry;

/// Migration v12: durable ContextRequest and ContextView records.
pub const CONTEXT_STORE_SCHEMA_V12: &str = "
CREATE TABLE IF NOT EXISTS context_requests (
  request_id     TEXT PRIMARY KEY,
  task_ref       TEXT NOT NULL,
  request_digest TEXT NOT NULL,
  canonical_json TEXT NOT NULL
) STRICT;
CREATE TRIGGER IF NOT EXISTS context_requests_append_only_update
BEFORE UPDATE ON context_requests
BEGIN SELECT RAISE(ABORT, 'append-only: context request is immutable'); END;
CREATE TRIGGER IF NOT EXISTS context_requests_append_only_delete
BEFORE DELETE ON context_requests
BEGIN SELECT RAISE(ABORT, 'append-only: context request is immutable'); END;

CREATE TABLE IF NOT EXISTS context_views (
  view_id        TEXT PRIMARY KEY,
  request_id     TEXT NOT NULL REFERENCES context_requests(request_id),
  view_digest    TEXT NOT NULL,
  canonical_json TEXT NOT NULL
) STRICT;
CREATE INDEX IF NOT EXISTS context_views_request_id
ON context_views (request_id);
CREATE TRIGGER IF NOT EXISTS context_views_append_only_update
BEFORE UPDATE ON context_views
BEGIN SELECT RAISE(ABORT, 'append-only: context view is immutable'); END;
CREATE TRIGGER IF NOT EXISTS context_views_append_only_delete
BEFORE DELETE ON context_views
BEGIN SELECT RAISE(ABORT, 'append-only: context view is immutable'); END;
";

/// The version-12 durable Context persistence migration entry.
pub fn context_store_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(12, CONTEXT_STORE_SCHEMA_V12)
}

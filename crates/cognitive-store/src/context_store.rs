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

/// Migration v13: daemon-admitted workspace Context source metadata and body.
pub const WORKSPACE_CONTEXT_SOURCE_SCHEMA_V13: &str = "
CREATE TABLE IF NOT EXISTS workspace_context_sources (
  source_id TEXT PRIMARY KEY,
  source_digest TEXT NOT NULL,
  tenant_id TEXT NOT NULL,
  owner_ref TEXT NOT NULL,
  resource_scope TEXT NOT NULL,
  conversation_ref TEXT,
  role TEXT NOT NULL CHECK (role IN ('control','authoritative_state','evidence','working','untrusted_input')),
  trust_level TEXT NOT NULL CHECK (trust_level IN ('control','authoritative','verified','untrusted')),
  representation TEXT NOT NULL CHECK (representation IN ('structured','text','binary_ref')),
  provenance_ref TEXT NOT NULL,
  content_bytes INTEGER NOT NULL CHECK (content_bytes >= 0),
  content_tokens INTEGER CHECK (content_tokens IS NULL OR content_tokens >= 0),
  canonical_json TEXT NOT NULL,
  CHECK ((role <> 'control' OR trust_level = 'control') AND (role <> 'authoritative_state' OR trust_level IN ('authoritative','control')) AND (trust_level <> 'untrusted' OR role IN ('untrusted_input','evidence','working')))
) STRICT;
CREATE INDEX IF NOT EXISTS workspace_context_sources_metadata_lookup
ON workspace_context_sources (tenant_id, resource_scope, conversation_ref, source_id);
CREATE TRIGGER IF NOT EXISTS workspace_context_sources_append_only_update
BEFORE UPDATE ON workspace_context_sources
BEGIN SELECT RAISE(ABORT, 'append-only: workspace Context source is immutable'); END;
CREATE TRIGGER IF NOT EXISTS workspace_context_sources_append_only_delete
BEFORE DELETE ON workspace_context_sources
BEGIN SELECT RAISE(ABORT, 'append-only: workspace Context source is immutable'); END;
";

pub fn workspace_context_source_migration_entry() -> MigrationPlanEntry {
    MigrationPlanEntry::new(13, WORKSPACE_CONTEXT_SOURCE_SCHEMA_V13)
}

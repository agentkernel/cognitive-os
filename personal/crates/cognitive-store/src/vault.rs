//! Personal-private Markdown Vault (P11-T10, authority migration v32).
//!
//! Import → rights/provenance → parse/index → conflict. The index is
//! rebuildable and is not Memory FTS. Vault files are not Project authority
//! (research notes stay here; confirmed decisions stay on draft/authority
//! SQLite). Conversation archive (P11-T05) is not Vault. Artifact CAS may
//! hold optional blobs; Vault metadata lives in these tables.

use crate::project_aggregate::{ConfirmCaller, ProjectAggregateError};
use crate::sqlite::SqliteAuthorityStore;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Mutex};

/// Personal-private Markdown Vault envelope (P11-T10).
pub const VAULT_PROJECTION_ID: &str = "cognitiveos.personal.markdown-vault/0.1";
/// Single-import body ceiling. Host filesystem E2E is out of this slice.
pub const VAULT_BODY_LIMIT: usize = 65_536;
/// Context assembly budget for the documented inject-order helper.
pub const VAULT_CONTEXT_BUDGET_BYTES: usize = 4_096;

/// Codex-inspired inject order. Task contract and fixed decisions are
/// Project-authority layers; Vault may only fill sourced excerpts, summaries,
/// and older narrative. Over-limit drops from the tail.
pub const CONTEXT_INJECT_ORDER: [&str; 5] = [
    "task-contract",
    "fixed-decision",
    "sourced-excerpt",
    "summary",
    "older-narrative",
];

const RIGHTS_CLASSES: &[&str] = &[
    "owner-owned",
    "licensed",
    "open-license",
    "public-domain",
    "citation-only",
];
const SOURCE_KINDS: &[&str] = &["markdown-file", "owner-paste"];

/// Authority migration v32: Vault documents, rebuildable index, conflicts.
///
/// The CHECK lists only layers Vault may persist. Inject-order labels for the
/// Task-contract and fixed-decision layers stay in `CONTEXT_INJECT_ORDER`
/// (Rust/HTTP) and are never stored: Vault must not persist authority layers,
/// and the hyphenated Task-contract token contains the secret-shape substring
/// that P8-T13 scans for in raw authority SQLite bytes.
pub const VAULT_SCHEMA_V32: &str = "
CREATE TABLE p11_vault_document (
  document_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  relative_path TEXT NOT NULL,
  rights_class TEXT NOT NULL,
  provenance_json TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  content_digest TEXT NOT NULL CHECK (length(content_digest) = 64),
  body_markdown TEXT NOT NULL,
  cas_ref TEXT,
  is_authority INTEGER NOT NULL DEFAULT 0 CHECK (is_authority = 0),
  imported_at INTEGER NOT NULL,
  UNIQUE (project_id, relative_path, content_digest)
) STRICT;
CREATE INDEX p11_vault_document_scope
  ON p11_vault_document(project_id, relative_path, imported_at);
CREATE TABLE p11_vault_index_entry (
  entry_id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL REFERENCES p11_vault_document(document_id),
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  chunk_ordinal INTEGER NOT NULL,
  excerpt TEXT NOT NULL,
  layer TEXT NOT NULL CHECK (layer IN (
    'sourced-excerpt','summary','older-narrative'
  )),
  rebuilt_at INTEGER NOT NULL
) STRICT;
CREATE INDEX p11_vault_index_scope
  ON p11_vault_index_entry(project_id, layer, chunk_ordinal);
CREATE TABLE p11_vault_conflict (
  conflict_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES p11_project(project_id),
  relative_path TEXT NOT NULL,
  incumbent_document_id TEXT NOT NULL REFERENCES p11_vault_document(document_id),
  incoming_document_id TEXT NOT NULL REFERENCES p11_vault_document(document_id),
  incoming_digest TEXT NOT NULL CHECK (length(incoming_digest) = 64),
  recorded_at INTEGER NOT NULL,
  resolution TEXT NOT NULL CHECK (resolution IN (
    'open','owner-chose-incumbent','owner-chose-incoming'
  ))
) STRICT;
CREATE INDEX p11_vault_conflict_scope
  ON p11_vault_conflict(project_id, relative_path, recorded_at);
";

/// v32 migration entry.
pub fn vault_migration_entry() -> crate::migration::MigrationPlanEntry {
    crate::migration::MigrationPlanEntry::new(32, VAULT_SCHEMA_V32)
}

/// Import input (Clippy-safe argument bundle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultImportSpec<'a> {
    pub project_id: &'a str,
    pub relative_path: &'a str,
    pub rights_class: &'a str,
    pub provenance_json: &'a str,
    pub source_kind: &'a str,
    pub body: &'a str,
    pub cas_ref: Option<&'a str>,
    pub conflict_policy: Option<&'a str>,
    pub now_ms: i64,
}

/// Index/query input. Cross-project caller is retrieval overreach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VaultReadSpec<'a> {
    pub caller_project_id: &'a str,
    pub target_project_id: &'a str,
}

/// Persisted Vault document metadata. Body is research notes, not authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultDocument {
    pub document_id: String,
    pub project_id: String,
    pub relative_path: String,
    pub rights_class: String,
    pub provenance_json: String,
    pub source_kind: String,
    pub content_digest: String,
    pub body_markdown: String,
    pub cas_ref: Option<String>,
    pub is_authority: i64,
    pub imported_at: i64,
}

/// One rebuildable index chunk. Not a Memory FTS row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultIndexEntry {
    pub entry_id: String,
    pub document_id: String,
    pub project_id: String,
    pub chunk_ordinal: i64,
    pub excerpt: String,
    pub layer: String,
    pub rebuilt_at: i64,
}

/// Index excerpt plus the P13-T07 label surface. Files stay non-authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultLabeledEntry {
    pub entry_id: String,
    pub document_id: String,
    pub relative_path: String,
    pub excerpt: String,
    pub layer: String,
    pub provenance_source_uri: String,
    pub rights_class: String,
    pub freshness: String,
    pub exclusion: String,
    pub exclusion_reason: String,
    pub untrusted_observation: bool,
    pub is_authority: bool,
}

/// Import visibility: a stored document remains visible when not indexed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultDocumentStatus {
    pub document_id: String,
    pub relative_path: String,
    pub provenance_source_uri: String,
    pub index_status: String,
}

/// Fail-closed conflict record. Last-write-wins without this row is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultConflict {
    pub conflict_id: String,
    pub project_id: String,
    pub relative_path: String,
    pub incumbent_document_id: String,
    pub incoming_document_id: String,
    pub incoming_digest: String,
    pub recorded_at: i64,
    pub resolution: String,
}

/// Documented Context inject plan. Vault never fills authority layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextInjectPlan {
    pub order: Vec<String>,
    pub excerpts: Vec<VaultIndexEntry>,
    pub dropped_layers: Vec<String>,
}

/// Personal-private Markdown Vault on the authority writer.
#[derive(Clone)]
pub struct VaultStore {
    conn: Arc<Mutex<Connection>>,
}

impl VaultStore {
    /// Share the daemon-owned authority writer.
    pub fn from_authority_store(store: &SqliteAuthorityStore) -> Self {
        Self {
            conn: Arc::clone(&store.conn),
        }
    }

    /// Open the authority database path (tests / CLI-free helpers).
    pub fn open_path(path: &std::path::Path) -> Result<Self, ProjectAggregateError> {
        let conn = Connection::open(path).map_err(unavailable("open"))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;",
        )
        .map_err(unavailable("pragma"))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, ProjectAggregateError> {
        self.conn
            .lock()
            .map_err(|_| ProjectAggregateError::Unavailable {
                detail: "authority writer lock poisoned".to_owned(),
            })
    }

    /// Import one Markdown file with rights and provenance.
    pub fn import(
        &self,
        caller: ConfirmCaller,
        spec: &VaultImportSpec<'_>,
    ) -> Result<String, ProjectAggregateError> {
        require_owner(caller)?;
        reject_secret_shape(spec.body)?;
        reject_secret_shape(spec.provenance_json)?;
        reject_secret_shape(spec.relative_path)?;
        reject_oversize_body(spec.body)?;
        reject_traversal(spec.relative_path)?;
        reject_conversation_as_vault(spec.source_kind)?;
        reject_cas_as_vault_file(spec.body, spec.cas_ref)?;
        require_rights(spec.rights_class)?;
        require_source_kind(spec.source_kind)?;
        require_provenance_object(spec.provenance_json)?;
        self.require_project(spec.project_id)?;
        let digest = digest_hex(spec.body.as_bytes());
        let conn = self.lock()?;
        if let Some(existing) =
            load_same_digest(&conn, spec.project_id, spec.relative_path, &digest)?
        {
            return Ok(existing);
        }
        let incumbent = load_incumbent(&conn, spec.project_id, spec.relative_path)?;
        if let Some(incumbent_id) = incumbent.as_ref() {
            match spec.conflict_policy {
                Some("record") => {}
                Some("last-write-wins") | None => {
                    return Err(ProjectAggregateError::Invalid {
                        detail: "last-write-wins without a conflict record is rejected",
                    });
                }
                Some(_) => {
                    return Err(ProjectAggregateError::Invalid {
                        detail: "unknown vault conflict policy",
                    });
                }
            }
            let incoming_id = insert_document(&conn, spec, &digest)?;
            insert_conflict(
                &conn,
                spec.project_id,
                spec.relative_path,
                incumbent_id,
                &incoming_id,
                &digest,
                spec.now_ms,
            )?;
            return Ok(incoming_id);
        }
        insert_document(&conn, spec, &digest)
    }

    /// Rebuild the derived index from stored documents. Not Memory FTS.
    pub fn rebuild_index(
        &self,
        caller: ConfirmCaller,
        project_id: &str,
        now_ms: i64,
    ) -> Result<usize, ProjectAggregateError> {
        require_owner(caller)?;
        self.require_project(project_id)?;
        let conn = self.lock()?;
        conn.execute(
            "DELETE FROM p11_vault_index_entry WHERE project_id = ?1",
            params![project_id],
        )
        .map_err(unavailable("clear vault index"))?;
        let documents = load_documents(&conn, project_id)?;
        let mut written = 0usize;
        for document in documents {
            for (ordinal, excerpt) in chunk_markdown(&document.body_markdown)
                .into_iter()
                .enumerate()
            {
                let entry_id = next_id("vault-idx")?;
                conn.execute(
                    "INSERT INTO p11_vault_index_entry (
                       entry_id, document_id, project_id, chunk_ordinal, excerpt, layer, rebuilt_at
                     ) VALUES (?1, ?2, ?3, ?4, ?5, 'sourced-excerpt', ?6)",
                    params![
                        entry_id,
                        document.document_id,
                        project_id,
                        ordinal as i64,
                        excerpt,
                        now_ms
                    ],
                )
                .map_err(unavailable("insert vault index"))?;
                written += 1;
            }
        }
        Ok(written)
    }

    /// Labeled index query. Cross-project caller is retrieval overreach.
    pub fn read_labeled_index(
        &self,
        spec: &VaultReadSpec<'_>,
    ) -> Result<Vec<VaultLabeledEntry>, ProjectAggregateError> {
        reject_cross_project(spec.caller_project_id, spec.target_project_id)?;
        let documents = {
            let conn = self.lock()?;
            load_documents(&conn, spec.target_project_id)?
        };
        let entries = self.read_index(spec)?;
        let current_by_path = current_document_ids(&documents);
        Ok(entries
            .into_iter()
            .filter_map(|entry| {
                let document = documents
                    .iter()
                    .find(|row| row.document_id == entry.document_id)?;
                Some(label_entry(
                    &entry,
                    document,
                    current_by_path.contains(&entry.document_id),
                ))
            })
            .collect())
    }

    /// Stored documents stay visible when the derived index has not rebuilt.
    pub fn list_document_statuses(
        &self,
        spec: &VaultReadSpec<'_>,
    ) -> Result<Vec<VaultDocumentStatus>, ProjectAggregateError> {
        reject_cross_project(spec.caller_project_id, spec.target_project_id)?;
        let conn = self.lock()?;
        let documents = load_documents(&conn, spec.target_project_id)?;
        let mut statuses = Vec::new();
        for document in documents {
            let indexed: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM p11_vault_index_entry WHERE document_id = ?1",
                    params![document.document_id],
                    |row| row.get(0),
                )
                .map_err(unavailable("count vault index for document"))?;
            statuses.push(VaultDocumentStatus {
                document_id: document.document_id,
                relative_path: document.relative_path,
                provenance_source_uri: provenance_source_uri(&document.provenance_json),
                index_status: if indexed > 0 {
                    "indexed".to_owned()
                } else {
                    "not-indexed".to_owned()
                },
            });
        }
        Ok(statuses)
    }

    /// Scoped index query. Cross-project caller is retrieval overreach.
    pub fn read_index(
        &self,
        spec: &VaultReadSpec<'_>,
    ) -> Result<Vec<VaultIndexEntry>, ProjectAggregateError> {
        reject_cross_project(spec.caller_project_id, spec.target_project_id)?;
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT entry_id, document_id, project_id, chunk_ordinal, excerpt, layer, rebuilt_at
                 FROM p11_vault_index_entry
                 WHERE project_id = ?1
                 ORDER BY layer, chunk_ordinal, entry_id",
            )
            .map_err(unavailable("prepare vault index"))?;
        let rows = statement
            .query_map(params![spec.target_project_id], map_index_entry)
            .map_err(unavailable("query vault index"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect vault index"))
    }

    /// Scoped conflict query. Cross-project caller is retrieval overreach.
    pub fn list_conflicts(
        &self,
        spec: &VaultReadSpec<'_>,
    ) -> Result<Vec<VaultConflict>, ProjectAggregateError> {
        reject_cross_project(spec.caller_project_id, spec.target_project_id)?;
        let conn = self.lock()?;
        let mut statement = conn
            .prepare(
                "SELECT conflict_id, project_id, relative_path, incumbent_document_id,
                        incoming_document_id, incoming_digest, recorded_at, resolution
                 FROM p11_vault_conflict
                 WHERE project_id = ?1
                 ORDER BY recorded_at, conflict_id",
            )
            .map_err(unavailable("prepare vault conflicts"))?;
        let rows = statement
            .query_map(params![spec.target_project_id], map_conflict)
            .map_err(unavailable("query vault conflicts"))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(unavailable("collect vault conflicts"))
    }

    /// Files cannot confirm or apply Project authority.
    pub fn apply_as_project_authority(
        &self,
        _document_id: &str,
    ) -> Result<(), ProjectAggregateError> {
        Err(ProjectAggregateError::Invalid {
            detail: "vault files are not Project authority",
        })
    }

    /// Memory admission cannot swallow Vault files as authority.
    pub fn admit_as_memory(&self, _document_id: &str) -> Result<(), ProjectAggregateError> {
        Err(ProjectAggregateError::Invalid {
            detail: "vault files cannot enter Memory admission as authority",
        })
    }

    /// Documented Context inject-order helper. HTTP import/index is the caller.
    pub fn assemble_context_inject_order(
        &self,
        spec: &VaultReadSpec<'_>,
    ) -> Result<ContextInjectPlan, ProjectAggregateError> {
        let entries = self.read_index(spec)?;
        let mut kept = Vec::new();
        let mut dropped = Vec::new();
        let mut used = 0usize;
        for layer in CONTEXT_INJECT_ORDER {
            if layer == "task-contract" || layer == "fixed-decision" {
                continue;
            }
            let layer_entries: Vec<VaultIndexEntry> = entries
                .iter()
                .filter(|entry| entry.layer == layer)
                .cloned()
                .collect();
            let layer_bytes: usize = layer_entries.iter().map(|entry| entry.excerpt.len()).sum();
            if used + layer_bytes > VAULT_CONTEXT_BUDGET_BYTES && !kept.is_empty() {
                dropped.push(layer.to_owned());
                continue;
            }
            used += layer_bytes;
            kept.extend(layer_entries);
        }
        Ok(ContextInjectPlan {
            order: CONTEXT_INJECT_ORDER
                .iter()
                .map(|layer| (*layer).to_owned())
                .collect(),
            excerpts: kept,
            dropped_layers: dropped,
        })
    }

    /// Count Memory FTS rows. Vault rebuild must not write this table.
    pub fn memory_fts_row_count(&self) -> Result<i64, ProjectAggregateError> {
        let conn = self.lock()?;
        conn.query_row("SELECT count(*) FROM memory_search_fts", [], |row| {
            row.get(0)
        })
        .map_err(unavailable("count memory fts"))
    }

    fn require_project(&self, project_id: &str) -> Result<(), ProjectAggregateError> {
        let conn = self.lock()?;
        let found: Option<String> = conn
            .query_row(
                "SELECT project_id FROM p11_project WHERE project_id = ?1",
                params![project_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(unavailable("lookup project"))?;
        found.map(|_| ()).ok_or(ProjectAggregateError::NotFound {
            detail: "project not found",
        })
    }
}

fn insert_document(
    conn: &Connection,
    spec: &VaultImportSpec<'_>,
    digest: &str,
) -> Result<String, ProjectAggregateError> {
    let document_id = next_id("vault-doc")?;
    conn.execute(
        "INSERT INTO p11_vault_document (
           document_id, project_id, relative_path, rights_class, provenance_json,
           source_kind, content_digest, body_markdown, cas_ref, is_authority, imported_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10)",
        params![
            document_id,
            spec.project_id,
            spec.relative_path,
            spec.rights_class,
            spec.provenance_json,
            spec.source_kind,
            digest,
            spec.body,
            spec.cas_ref,
            spec.now_ms
        ],
    )
    .map_err(unavailable("insert vault document"))?;
    Ok(document_id)
}

fn insert_conflict(
    conn: &Connection,
    project_id: &str,
    relative_path: &str,
    incumbent_id: &str,
    incoming_id: &str,
    incoming_digest: &str,
    now_ms: i64,
) -> Result<(), ProjectAggregateError> {
    let conflict_id = next_id("vault-cfl")?;
    conn.execute(
        "INSERT INTO p11_vault_conflict (
           conflict_id, project_id, relative_path, incumbent_document_id,
           incoming_document_id, incoming_digest, recorded_at, resolution
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'open')",
        params![
            conflict_id,
            project_id,
            relative_path,
            incumbent_id,
            incoming_id,
            incoming_digest,
            now_ms
        ],
    )
    .map_err(unavailable("insert vault conflict"))?;
    Ok(())
}

fn load_same_digest(
    conn: &Connection,
    project_id: &str,
    relative_path: &str,
    digest: &str,
) -> Result<Option<String>, ProjectAggregateError> {
    conn.query_row(
        "SELECT document_id FROM p11_vault_document
         WHERE project_id = ?1 AND relative_path = ?2 AND content_digest = ?3",
        params![project_id, relative_path, digest],
        |row| row.get(0),
    )
    .optional()
    .map_err(unavailable("lookup vault digest"))
}

fn load_incumbent(
    conn: &Connection,
    project_id: &str,
    relative_path: &str,
) -> Result<Option<String>, ProjectAggregateError> {
    conn.query_row(
        "SELECT document_id FROM p11_vault_document
         WHERE project_id = ?1 AND relative_path = ?2
         ORDER BY imported_at DESC, document_id DESC
         LIMIT 1",
        params![project_id, relative_path],
        |row| row.get(0),
    )
    .optional()
    .map_err(unavailable("lookup vault incumbent"))
}

fn load_documents(
    conn: &Connection,
    project_id: &str,
) -> Result<Vec<VaultDocument>, ProjectAggregateError> {
    let mut statement = conn
        .prepare(
            "SELECT document_id, project_id, relative_path, rights_class, provenance_json,
                    source_kind, content_digest, body_markdown, cas_ref, is_authority, imported_at
             FROM p11_vault_document
             WHERE project_id = ?1
             ORDER BY imported_at, document_id",
        )
        .map_err(unavailable("prepare vault documents"))?;
    let rows = statement
        .query_map(params![project_id], map_document)
        .map_err(unavailable("query vault documents"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(unavailable("collect vault documents"))
}

fn map_document(row: &rusqlite::Row<'_>) -> rusqlite::Result<VaultDocument> {
    Ok(VaultDocument {
        document_id: row.get(0)?,
        project_id: row.get(1)?,
        relative_path: row.get(2)?,
        rights_class: row.get(3)?,
        provenance_json: row.get(4)?,
        source_kind: row.get(5)?,
        content_digest: row.get(6)?,
        body_markdown: row.get(7)?,
        cas_ref: row.get(8)?,
        is_authority: row.get(9)?,
        imported_at: row.get(10)?,
    })
}

fn map_index_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<VaultIndexEntry> {
    Ok(VaultIndexEntry {
        entry_id: row.get(0)?,
        document_id: row.get(1)?,
        project_id: row.get(2)?,
        chunk_ordinal: row.get(3)?,
        excerpt: row.get(4)?,
        layer: row.get(5)?,
        rebuilt_at: row.get(6)?,
    })
}

fn map_conflict(row: &rusqlite::Row<'_>) -> rusqlite::Result<VaultConflict> {
    Ok(VaultConflict {
        conflict_id: row.get(0)?,
        project_id: row.get(1)?,
        relative_path: row.get(2)?,
        incumbent_document_id: row.get(3)?,
        incoming_document_id: row.get(4)?,
        incoming_digest: row.get(5)?,
        recorded_at: row.get(6)?,
        resolution: row.get(7)?,
    })
}

fn chunk_markdown(body: &str) -> Vec<String> {
    let chunks: Vec<String> = body
        .split("\n\n")
        .map(str::trim)
        .filter(|chunk| !chunk.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    if chunks.is_empty() && !body.trim().is_empty() {
        vec![body.trim().to_owned()]
    } else {
        chunks
    }
}

fn require_owner(caller: ConfirmCaller) -> Result<(), ProjectAggregateError> {
    match caller {
        ConfirmCaller::OwnerManagement => Ok(()),
        ConfirmCaller::TaskChannel | ConfirmCaller::Assistant => {
            Err(ProjectAggregateError::Forbidden {
                detail: "only owner management session may confirm or apply",
            })
        }
    }
}

fn reject_cross_project(caller: &str, target: &str) -> Result<(), ProjectAggregateError> {
    if caller == target && !caller.is_empty() {
        return Ok(());
    }
    Err(ProjectAggregateError::Forbidden {
        detail: "cross-project vault read rejected",
    })
}

fn reject_secret_shape(body: &str) -> Result<(), ProjectAggregateError> {
    let lowered = body.to_ascii_lowercase();
    if lowered.contains("sk-")
        || lowered.contains("bearer ")
        || lowered.contains("api_key")
        || lowered.contains("x-api-key")
        || lowered.contains("ssv1:")
    {
        return Err(ProjectAggregateError::Invalid {
            detail: "secret-shaped material is rejected at registration",
        });
    }
    Ok(())
}

fn reject_oversize_body(body: &str) -> Result<(), ProjectAggregateError> {
    if body.len() > VAULT_BODY_LIMIT {
        return Err(ProjectAggregateError::Invalid {
            detail: "vault import body exceeds the bounded ceiling",
        });
    }
    Ok(())
}

fn reject_traversal(relative_path: &str) -> Result<(), ProjectAggregateError> {
    let lowered = relative_path.replace('\\', "/");
    if relative_path.is_empty()
        || lowered.starts_with('/')
        || lowered.contains("..")
        || relative_path.contains(':')
        || relative_path.contains('\\')
    {
        return Err(ProjectAggregateError::Invalid {
            detail: "vault path traversal is rejected",
        });
    }
    Ok(())
}

fn reject_conversation_as_vault(source_kind: &str) -> Result<(), ProjectAggregateError> {
    if source_kind == "conversation-archive"
        || source_kind == "cognitiveos.personal.conversation-archive/0.1"
    {
        return Err(ProjectAggregateError::Invalid {
            detail: "conversation archive is not a Vault file",
        });
    }
    Ok(())
}

fn reject_cas_as_vault_file(
    body: &str,
    cas_ref: Option<&str>,
) -> Result<(), ProjectAggregateError> {
    if body.trim().is_empty() && cas_ref.is_some() {
        return Err(ProjectAggregateError::Invalid {
            detail: "artifact CAS is not a Vault file",
        });
    }
    if body.trim().is_empty() {
        return Err(ProjectAggregateError::Invalid {
            detail: "vault import requires markdown body metadata in the store",
        });
    }
    Ok(())
}

fn require_rights(rights_class: &str) -> Result<(), ProjectAggregateError> {
    if RIGHTS_CLASSES.contains(&rights_class) {
        return Ok(());
    }
    Err(ProjectAggregateError::Invalid {
        detail: "vault import requires an allowed rights class",
    })
}

fn require_source_kind(source_kind: &str) -> Result<(), ProjectAggregateError> {
    if SOURCE_KINDS.contains(&source_kind) {
        return Ok(());
    }
    Err(ProjectAggregateError::Invalid {
        detail: "vault import source_kind is not a Markdown Vault source",
    })
}

fn require_provenance_object(provenance_json: &str) -> Result<(), ProjectAggregateError> {
    let value: serde_json::Value =
        serde_json::from_str(provenance_json).map_err(|_| ProjectAggregateError::Invalid {
            detail: "vault import requires provenance JSON",
        })?;
    if value
        .get("source_uri")
        .and_then(serde_json::Value::as_str)
        .is_none()
    {
        return Err(ProjectAggregateError::Invalid {
            detail: "vault import requires provenance JSON",
        });
    }
    Ok(())
}

fn current_document_ids(documents: &[VaultDocument]) -> std::collections::HashSet<String> {
    let mut latest: std::collections::BTreeMap<&str, &VaultDocument> =
        std::collections::BTreeMap::new();
    for document in documents {
        match latest.get(document.relative_path.as_str()) {
            Some(incumbent)
                if incumbent.imported_at > document.imported_at
                    || (incumbent.imported_at == document.imported_at
                        && incumbent.document_id > document.document_id) => {}
            _ => {
                latest.insert(&document.relative_path, document);
            }
        }
    }
    latest
        .into_values()
        .map(|document| document.document_id.clone())
        .collect()
}

fn label_entry(
    entry: &VaultIndexEntry,
    document: &VaultDocument,
    current: bool,
) -> VaultLabeledEntry {
    let citation_only = document.rights_class == "citation-only";
    VaultLabeledEntry {
        entry_id: entry.entry_id.clone(),
        document_id: entry.document_id.clone(),
        relative_path: document.relative_path.clone(),
        excerpt: entry.excerpt.clone(),
        layer: entry.layer.clone(),
        provenance_source_uri: provenance_source_uri(&document.provenance_json),
        rights_class: document.rights_class.clone(),
        freshness: if current {
            "current".to_owned()
        } else {
            "superseded".to_owned()
        },
        exclusion: if citation_only {
            "excluded".to_owned()
        } else {
            "included".to_owned()
        },
        exclusion_reason: if citation_only {
            "citation-only".to_owned()
        } else {
            String::new()
        },
        untrusted_observation: citation_only,
        is_authority: document.is_authority != 0,
    }
}

fn provenance_source_uri(provenance_json: &str) -> String {
    serde_json::from_str::<serde_json::Value>(provenance_json)
        .ok()
        .and_then(|value| {
            value
                .get("source_uri")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

fn digest_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn next_id(prefix: &str) -> Result<String, ProjectAggregateError> {
    let generated = uuid::Uuid::now_v7().as_hyphenated().to_string();
    Ok(format!("{prefix}-{generated}"))
}

fn unavailable(operation: &'static str) -> impl Fn(rusqlite::Error) -> ProjectAggregateError {
    move |source| ProjectAggregateError::Unavailable {
        detail: format!("{operation}: {source}"),
    }
}

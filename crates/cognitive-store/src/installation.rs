//! Durable SQLite installation staging and commit adapter.
//!
//! This is deliberately a narrow KRN-owned port. It records the immutable
//! inputs that a later Lane-RUN authority commit must consume; it does not
//! grant a capability, change an AgentInstallation lifecycle state, or claim
//! package provenance verification. D-020 explicitly prohibits introducing a
//! sixth transition table for this purpose.

use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use thiserror::Error;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS installation_staging (
  package_ref          TEXT PRIMARY KEY,
  package_digest       TEXT NOT NULL,
  adapter_digest       TEXT NOT NULL,
  sandbox_digest       TEXT NOT NULL,
  compatibility_digest TEXT NOT NULL,
  source_mode          TEXT,
  operator_ref         TEXT,
  project_ref          TEXT,
  lockfile_digest      TEXT,
  verification_result  TEXT
) STRICT;

CREATE TABLE IF NOT EXISTS installations (
  package_ref          TEXT PRIMARY KEY,
  package_digest       TEXT NOT NULL,
  adapter_digest       TEXT NOT NULL,
  sandbox_digest       TEXT NOT NULL,
  compatibility_digest TEXT NOT NULL,
  source_mode          TEXT,
  operator_ref         TEXT,
  project_ref          TEXT,
  lockfile_digest      TEXT,
  verification_result  TEXT
) STRICT;

CREATE TRIGGER IF NOT EXISTS installations_append_only_update
BEFORE UPDATE ON installations
BEGIN SELECT RAISE(ABORT, 'append-only: committed installations are immutable'); END;

CREATE TRIGGER IF NOT EXISTS installations_append_only_delete
BEFORE DELETE ON installations
BEGIN SELECT RAISE(ABORT, 'append-only: committed installations are immutable'); END;
";

/// Errors from the local durable installation store.
///
/// These are adapter errors, not protocol error codes: no machine contract is
/// added by this KRN-only persistence slice (D-020).
#[derive(Debug, Error)]
pub enum InstallationStoreError {
    /// The candidate was incomplete and must not reach durable staging.
    #[error("invalid installation commit: {detail}")]
    InvalidCommit { detail: String },
    /// A stage/commit operation conflicted with the current durable contents.
    #[error("installation-store conflict: {detail}")]
    Conflict { detail: String },
    /// SQLite could not durably complete an operation; callers must fail closed.
    #[error("installation-store unavailable: {detail}")]
    Unavailable { detail: String },
}

/// Immutable source-admission evidence bound to a durable installation.
///
/// This is a KRN persistence carrier, not publisher provenance and not an
/// authorization capability. It keeps the Custom acknowledgement in the same
/// stage-to-commit transaction as the package and policy digests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallationEvidence {
    source_mode: String,
    operator_ref: String,
    project_ref: String,
    lockfile_digest: String,
    verification_result: String,
}

impl InstallationEvidence {
    /// Evidence for one explicitly acknowledged local project bundle.
    pub fn custom_user_provided(
        operator_ref: impl Into<String>,
        project_ref: impl Into<String>,
        lockfile_digest: impl Into<String>,
        verification_result: impl Into<String>,
    ) -> Result<Self, InstallationStoreError> {
        let evidence = Self {
            source_mode: "custom_user_provided".to_owned(),
            operator_ref: operator_ref.into(),
            project_ref: project_ref.into(),
            lockfile_digest: lockfile_digest.into(),
            verification_result: verification_result.into(),
        };
        if !evidence.operator_ref.starts_with("principal://")
            || !evidence.project_ref.starts_with("file://")
            || evidence.lockfile_digest.trim().is_empty()
            || evidence.verification_result.trim().is_empty()
        {
            return Err(InstallationStoreError::InvalidCommit {
                detail: "Custom evidence requires principal:// operator, file:// bundle, lockfile digest, and verification result".to_owned(),
            });
        }
        Ok(evidence)
    }

    pub fn source_mode(&self) -> &str {
        &self.source_mode
    }

    pub fn operator_ref(&self) -> &str {
        &self.operator_ref
    }

    pub fn project_ref(&self) -> &str {
        &self.project_ref
    }

    pub fn lockfile_digest(&self) -> &str {
        &self.lockfile_digest
    }

    pub fn verification_result(&self) -> &str {
        &self.verification_result
    }
}

/// Immutable evidence inputs for an eventual managed installation commit.
///
/// The record is intentionally authority-neutral. It proves only that the
/// supplied values crossed the store's staging/commit boundary; Lane-RUN must
/// still validate provenance, sandbox evidence, compatibility, and management
/// authority before it can create an `AgentInstallation`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstallationCommit {
    package_ref: String,
    package_digest: String,
    adapter_digest: String,
    sandbox_digest: String,
    compatibility_digest: String,
    evidence: Option<InstallationEvidence>,
}

impl InstallationCommit {
    /// Construct a complete, non-empty set of immutable installation inputs.
    pub fn new(
        package_ref: impl Into<String>,
        package_digest: impl Into<String>,
        adapter_digest: impl Into<String>,
        sandbox_digest: impl Into<String>,
        compatibility_digest: impl Into<String>,
    ) -> Result<Self, InstallationStoreError> {
        Self::new_optional_evidence(
            package_ref,
            package_digest,
            adapter_digest,
            sandbox_digest,
            compatibility_digest,
            None,
        )
    }

    /// Construct a durable record with source-admission evidence that must
    /// become visible atomically with the package commit.
    pub fn new_with_evidence(
        package_ref: impl Into<String>,
        package_digest: impl Into<String>,
        adapter_digest: impl Into<String>,
        sandbox_digest: impl Into<String>,
        compatibility_digest: impl Into<String>,
        evidence: InstallationEvidence,
    ) -> Result<Self, InstallationStoreError> {
        Self::new_optional_evidence(
            package_ref,
            package_digest,
            adapter_digest,
            sandbox_digest,
            compatibility_digest,
            Some(evidence),
        )
    }

    fn new_optional_evidence(
        package_ref: impl Into<String>,
        package_digest: impl Into<String>,
        adapter_digest: impl Into<String>,
        sandbox_digest: impl Into<String>,
        compatibility_digest: impl Into<String>,
        evidence: Option<InstallationEvidence>,
    ) -> Result<Self, InstallationStoreError> {
        let commit = Self {
            package_ref: package_ref.into(),
            package_digest: package_digest.into(),
            adapter_digest: adapter_digest.into(),
            sandbox_digest: sandbox_digest.into(),
            compatibility_digest: compatibility_digest.into(),
            evidence,
        };
        for (name, value) in [
            ("package_ref", &commit.package_ref),
            ("package_digest", &commit.package_digest),
            ("adapter_digest", &commit.adapter_digest),
            ("sandbox_digest", &commit.sandbox_digest),
            ("compatibility_digest", &commit.compatibility_digest),
        ] {
            if value.trim().is_empty() {
                return Err(InstallationStoreError::InvalidCommit {
                    detail: format!("{name} must not be empty"),
                });
            }
        }
        Ok(commit)
    }

    /// Stable package identity used for staging and eventual lookup.
    pub fn package_ref(&self) -> &str {
        &self.package_ref
    }

    /// Custom source evidence, if this record was committed through that mode.
    pub fn evidence(&self) -> Option<&InstallationEvidence> {
        self.evidence.as_ref()
    }
}

/// SQLite WAL store with atomic stage-to-commit visibility.
///
/// Committed rows are the only rows returned to a reader, and staging is never
/// promoted except by [`Self::commit`]. The installation authority invokes
/// [`Self::recover_interrupted_staging`] under its exclusive lifecycle lock;
/// ordinary reader handles never discard another writer's staging.
pub struct SqliteInstallationStore {
    conn: Mutex<Connection>,
}

impl SqliteInstallationStore {
    /// Open a durable installation store without exposing staging rows.
    pub fn open(path: &Path) -> Result<Self, InstallationStoreError> {
        let conn = Connection::open(path).map_err(|err| unavailable("open", err))?;
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
            .map_err(|err| unavailable("set journal_mode", err))?;
        if !journal_mode.eq_ignore_ascii_case("wal") {
            return Err(InstallationStoreError::Unavailable {
                detail: format!("installation database refused WAL mode: {journal_mode}"),
            });
        }
        conn.execute_batch(
            "PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;",
        )
        .map_err(|err| unavailable("set pragmas", err))?;
        conn.execute_batch(SCHEMA)
            .map_err(|err| unavailable("install schema", err))?;
        ensure_evidence_columns(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Discard durable but uncommitted staging after a confirmed interrupted
    /// installation attempt.
    ///
    /// Callers must hold their installation-lifecycle exclusion before calling
    /// this method. It is intentionally explicit so opening a reader cannot
    /// erase staging owned by a live writer.
    pub fn recover_interrupted_staging(&self) -> Result<(), InstallationStoreError> {
        let mut conn = self.lock()?;
        let recovery = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|err| unavailable("begin recovery", err))?;
        recovery
            .execute("DELETE FROM installation_staging", [])
            .map_err(|err| unavailable("discard interrupted staging", err))?;
        recovery
            .commit()
            .map_err(|err| unavailable("commit recovery", err))
    }

    /// Durably stage a complete candidate without making it externally visible.
    pub fn stage(&self, commit: &InstallationCommit) -> Result<(), InstallationStoreError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|err| unavailable("begin staging", err))?;
        let inserted = tx.execute(
            "INSERT INTO installation_staging
               (package_ref, package_digest, adapter_digest, sandbox_digest, compatibility_digest,
                source_mode, operator_ref, project_ref, lockfile_digest, verification_result)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            (
                &commit.package_ref,
                &commit.package_digest,
                &commit.adapter_digest,
                &commit.sandbox_digest,
                &commit.compatibility_digest,
                commit.evidence.as_ref().map(|e| e.source_mode.as_str()),
                commit.evidence.as_ref().map(|e| e.operator_ref.as_str()),
                commit.evidence.as_ref().map(|e| e.project_ref.as_str()),
                commit.evidence.as_ref().map(|e| e.lockfile_digest.as_str()),
                commit
                    .evidence
                    .as_ref()
                    .map(|e| e.verification_result.as_str()),
            ),
        );
        match inserted {
            Ok(_) => tx
                .commit()
                .map_err(|err| unavailable("commit staging", err)),
            Err(err) if is_constraint_violation(&err) => Err(InstallationStoreError::Conflict {
                detail: format!("package {} is already staged", commit.package_ref),
            }),
            Err(err) => Err(unavailable("stage installation", err)),
        }
    }

    /// Atomically promote a staged candidate to the immutable committed view.
    pub fn commit(&self, package_ref: &str) -> Result<(), InstallationStoreError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|err| unavailable("begin installation commit", err))?;
        let promoted = tx.execute(
                "INSERT INTO installations
                   (package_ref, package_digest, adapter_digest, sandbox_digest, compatibility_digest,
                    source_mode, operator_ref, project_ref, lockfile_digest, verification_result)
                 SELECT package_ref, package_digest, adapter_digest, sandbox_digest, compatibility_digest,
                        source_mode, operator_ref, project_ref, lockfile_digest, verification_result
                   FROM installation_staging WHERE package_ref = ?1",
                [package_ref],
            );
        let promoted = match promoted {
            Ok(promoted) => promoted,
            Err(err) if is_constraint_violation(&err) => {
                return Err(InstallationStoreError::Conflict {
                    detail: format!("package {package_ref} is already committed"),
                });
            }
            Err(err) => return Err(unavailable("promote staged installation", err)),
        };
        if promoted == 0 {
            return Err(InstallationStoreError::Conflict {
                detail: format!("no staged package {package_ref}"),
            });
        }
        tx.execute(
            "DELETE FROM installation_staging WHERE package_ref = ?1",
            [package_ref],
        )
        .map_err(|err| unavailable("clear committed staging", err))?;
        tx.commit()
            .map_err(|err| unavailable("commit installation", err))
    }

    /// Read only a fully committed record; staging is intentionally invisible.
    pub fn committed(
        &self,
        package_ref: &str,
    ) -> Result<Option<InstallationCommit>, InstallationStoreError> {
        let conn = self.lock()?;
        conn.query_row(
            "SELECT package_ref, package_digest, adapter_digest, sandbox_digest, compatibility_digest,
                    source_mode, operator_ref, project_ref, lockfile_digest, verification_result
               FROM installations WHERE package_ref = ?1",
            [package_ref],
            |row| {
                let source_mode: Option<String> = row.get(5)?;
                let operator_ref: Option<String> = row.get(6)?;
                let project_ref: Option<String> = row.get(7)?;
                let lockfile_digest: Option<String> = row.get(8)?;
                let verification_result: Option<String> = row.get(9)?;
                let evidence = match (
                    source_mode,
                    operator_ref,
                    project_ref,
                    lockfile_digest,
                    verification_result,
                ) {
                    (None, None, None, None, None) => None,
                    (Some(source_mode), Some(operator_ref), Some(project_ref), Some(lockfile_digest), Some(verification_result)) => Some(InstallationEvidence {
                        source_mode,
                        operator_ref,
                        project_ref,
                        lockfile_digest,
                        verification_result,
                    }),
                    _ => return Err(rusqlite::Error::InvalidQuery),
                };
                Ok(InstallationCommit {
                    package_ref: row.get(0)?,
                    package_digest: row.get(1)?,
                    adapter_digest: row.get(2)?,
                    sandbox_digest: row.get(3)?,
                    compatibility_digest: row.get(4)?,
                    evidence,
                })
            },
        )
        .optional()
        .map_err(|err| unavailable("read committed installation", err))
    }

    /// Return the number of non-visible staging rows, for recovery assertions.
    pub fn staging_count(&self) -> Result<usize, InstallationStoreError> {
        let conn = self.lock()?;
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM installation_staging", [], |row| {
                row.get(0)
            })
            .map_err(|err| unavailable("count staging", err))?;
        usize::try_from(count).map_err(|err| InstallationStoreError::Unavailable {
            detail: format!("invalid staging count: {err}"),
        })
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, InstallationStoreError> {
        self.conn
            .lock()
            .map_err(|_| InstallationStoreError::Unavailable {
                detail: "installation connection poisoned".to_owned(),
            })
    }
}

fn ensure_evidence_columns(conn: &Connection) -> Result<(), InstallationStoreError> {
    for table in ["installation_staging", "installations"] {
        for column in [
            "source_mode TEXT",
            "operator_ref TEXT",
            "project_ref TEXT",
            "lockfile_digest TEXT",
            "verification_result TEXT",
        ] {
            let statement = format!("ALTER TABLE {table} ADD COLUMN {column}");
            match conn.execute(&statement, []) {
                Ok(_) => {}
                Err(err) if err.to_string().contains("duplicate column name") => {}
                Err(err) => return Err(unavailable("migrate installation evidence", err)),
            }
        }
    }
    Ok(())
}

fn unavailable(what: &str, err: impl std::fmt::Display) -> InstallationStoreError {
    InstallationStoreError::Unavailable {
        detail: format!("{what}: {err}"),
    }
}

fn is_constraint_violation(err: &rusqlite::Error) -> bool {
    matches!(
        err,
        rusqlite::Error::SqliteFailure(failure, _)
            if failure.code == rusqlite::ErrorCode::ConstraintViolation
    )
}

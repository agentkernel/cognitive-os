//! Versioned SQLite schema migrations for Personal data-layout validation.
//!
//! This module is intentionally adapter-local: it records schema history and
//! validates replay safety, but does not alter authority transition semantics
//! or create a new machine contract.

use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use thiserror::Error;

const MIGRATION_METADATA_SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS schema_migrations (
  version    INTEGER PRIMARY KEY CHECK (version > 0),
  digest     TEXT NOT NULL,
  applied_at TEXT NOT NULL
) STRICT;
";

/// One immutable schema change in the ordered migration plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationPlanEntry {
    version: i64,
    digest: String,
    sql: &'static str,
}

impl MigrationPlanEntry {
    /// Creates one migration entry with a SHA-256 digest derived from its SQL.
    /// Validation happens before any database is copied or modified, so an
    /// invalid plan fails without a side effect.
    pub fn new(version: i64, sql: &'static str) -> Self {
        Self {
            version,
            digest: calculate_migration_digest(sql),
            sql,
        }
    }
}

/// The permitted migration execution paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MigrationExecutionMode {
    /// Copy the source database and run the plan only on that scratch copy.
    DryRun { scratch_database_path: PathBuf },
    /// Create a durable pre-migration backup, then atomically apply the plan.
    Apply { backup_database_path: PathBuf },
}

/// Facts from one migration invocation. This is a local report, not release
/// evidence and not an authority projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationExecutionReport {
    applied_versions: Vec<i64>,
    backup_database_path: Option<PathBuf>,
}

impl MigrationExecutionReport {
    /// Versions newly recorded by this invocation, in plan order.
    pub fn applied_versions(&self) -> &[i64] {
        &self.applied_versions
    }

    /// The backup used for an apply operation; dry runs never expose one.
    pub fn backup_database_path(&self) -> Option<&Path> {
        self.backup_database_path.as_deref()
    }
}

/// Fail-closed outcomes from the migration adapter.
#[derive(Debug, Error)]
pub enum SqliteMigrationError {
    /// The supplied immutable plan is internally inconsistent.
    #[error("invalid migration plan: {detail}")]
    InvalidPlan { detail: String },
    /// An existing database has a migration record absent from the binary.
    #[error("database records unknown migration version {version}")]
    UnknownRecordedVersion { version: i64 },
    /// A migration's immutable digest changed after it was applied.
    #[error("migration digest mismatch at version {version}: expected {expected}, found {found}")]
    DigestMismatch {
        version: i64,
        expected: String,
        found: String,
    },
    /// A requested scratch or backup destination is unsafe to overwrite.
    #[error("migration copy destination is unsafe: {detail}")]
    UnsafeCopyDestination { detail: String },
    /// SQLite rejected an operation. The source database is unchanged when
    /// this is raised while applying a migration transaction.
    #[error("sqlite migration operation failed during {operation}: {source}")]
    Sqlite {
        operation: &'static str,
        #[source]
        source: rusqlite::Error,
    },
}

/// Runs a validated plan either against a scratch copy or the source database.
///
/// `Apply` writes a pre-migration SQLite backup before touching the source.
/// Every schema change and its metadata row share one immediate transaction;
/// SQL failure rolls both back. `DryRun` never installs metadata in the source.
pub fn execute_sqlite_migration_plan(
    database_path: &Path,
    migration_plan: &[MigrationPlanEntry],
    execution_mode: MigrationExecutionMode,
) -> Result<MigrationExecutionReport, SqliteMigrationError> {
    validate_migration_plan(migration_plan)?;

    match execution_mode {
        MigrationExecutionMode::DryRun {
            scratch_database_path,
        } => {
            copy_database(database_path, &scratch_database_path)?;
            let applied_versions = apply_migration_plan(&scratch_database_path, migration_plan)?;
            Ok(MigrationExecutionReport {
                applied_versions,
                backup_database_path: None,
            })
        }
        MigrationExecutionMode::Apply {
            backup_database_path,
        } => {
            copy_database(database_path, &backup_database_path)?;
            let applied_versions = apply_migration_plan(database_path, migration_plan)?;
            Ok(MigrationExecutionReport {
                applied_versions,
                backup_database_path: Some(backup_database_path),
            })
        }
    }
}

fn validate_migration_plan(
    migration_plan: &[MigrationPlanEntry],
) -> Result<(), SqliteMigrationError> {
    let mut previous_version = 0;
    for migration in migration_plan {
        if migration.version <= previous_version {
            return Err(SqliteMigrationError::InvalidPlan {
                detail: "versions must be strictly increasing positive integers".to_owned(),
            });
        }
        if migration.sql.trim().is_empty() {
            return Err(SqliteMigrationError::InvalidPlan {
                detail: format!("version {} has an empty SQL body", migration.version),
            });
        }
        let calculated_digest = calculate_migration_digest(migration.sql);
        if migration.digest != calculated_digest {
            return Err(SqliteMigrationError::InvalidPlan {
                detail: format!(
                    "version {} digest does not match its SQL body",
                    migration.version
                ),
            });
        }
        previous_version = migration.version;
    }
    Ok(())
}

fn copy_database(source_path: &Path, destination_path: &Path) -> Result<(), SqliteMigrationError> {
    if source_path == destination_path {
        return Err(SqliteMigrationError::UnsafeCopyDestination {
            detail: "source and copy destination must differ".to_owned(),
        });
    }
    if destination_path.exists() {
        return Err(SqliteMigrationError::UnsafeCopyDestination {
            detail: format!("destination already exists: {}", destination_path.display()),
        });
    }
    if !source_path.exists() {
        return Err(SqliteMigrationError::UnsafeCopyDestination {
            detail: format!("source database does not exist: {}", source_path.display()),
        });
    }

    let source_connection =
        Connection::open(source_path).map_err(|source| SqliteMigrationError::Sqlite {
            operation: "open source database for copy",
            source,
        })?;
    source_connection
        .execute_batch("PRAGMA wal_checkpoint(FULL);")
        .map_err(|source| SqliteMigrationError::Sqlite {
            operation: "checkpoint source database before copy",
            source,
        })?;
    let destination_database_string = destination_path.to_string_lossy();
    source_connection
        .execute("VACUUM INTO ?1", [destination_database_string.as_ref()])
        .map_err(|source| SqliteMigrationError::Sqlite {
            operation: "create consistent database copy",
            source,
        })?;
    Ok(())
}

fn apply_migration_plan(
    database_path: &Path,
    migration_plan: &[MigrationPlanEntry],
) -> Result<Vec<i64>, SqliteMigrationError> {
    let mut connection =
        Connection::open(database_path).map_err(|source| SqliteMigrationError::Sqlite {
            operation: "open database for migration",
            source,
        })?;
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|source| SqliteMigrationError::Sqlite {
            operation: "begin migration transaction",
            source,
        })?;
    transaction
        .execute_batch(MIGRATION_METADATA_SCHEMA)
        .map_err(|source| SqliteMigrationError::Sqlite {
            operation: "install migration metadata schema",
            source,
        })?;

    validate_recorded_migrations(&transaction, migration_plan)?;
    let mut applied_versions = Vec::new();
    for migration in migration_plan {
        let recorded_digest = transaction
            .query_row(
                "SELECT digest FROM schema_migrations WHERE version = ?1",
                [migration.version],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|source| SqliteMigrationError::Sqlite {
                operation: "read migration metadata",
                source,
            })?;
        if recorded_digest.is_some() {
            continue;
        }

        transaction.execute_batch(migration.sql).map_err(|source| {
            SqliteMigrationError::Sqlite {
                operation: "execute migration SQL",
                source,
            }
        })?;
        transaction
            .execute(
                "INSERT INTO schema_migrations (version, digest, applied_at)
                 VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
                (migration.version, &migration.digest),
            )
            .map_err(|source| SqliteMigrationError::Sqlite {
                operation: "record applied migration",
                source,
            })?;
        applied_versions.push(migration.version);
    }

    let integrity_result = transaction
        .query_row("PRAGMA quick_check", [], |row| row.get::<_, String>(0))
        .map_err(|source| SqliteMigrationError::Sqlite {
            operation: "verify migrated database integrity",
            source,
        })?;
    if integrity_result != "ok" {
        return Err(SqliteMigrationError::InvalidPlan {
            detail: format!("database integrity check failed after migration: {integrity_result}"),
        });
    }
    transaction
        .commit()
        .map_err(|source| SqliteMigrationError::Sqlite {
            operation: "commit migration transaction",
            source,
        })?;
    Ok(applied_versions)
}

fn validate_recorded_migrations(
    transaction: &rusqlite::Transaction<'_>,
    migration_plan: &[MigrationPlanEntry],
) -> Result<(), SqliteMigrationError> {
    let mut recorded_migrations = transaction
        .prepare("SELECT version, digest FROM schema_migrations ORDER BY version")
        .map_err(|source| SqliteMigrationError::Sqlite {
            operation: "prepare migration metadata query",
            source,
        })?;
    let recorded_rows = recorded_migrations
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|source| SqliteMigrationError::Sqlite {
            operation: "query migration metadata",
            source,
        })?;
    for recorded_row in recorded_rows {
        let (version, found_digest) =
            recorded_row.map_err(|source| SqliteMigrationError::Sqlite {
                operation: "read migration metadata row",
                source,
            })?;
        let Some(planned_migration) = migration_plan
            .iter()
            .find(|migration| migration.version == version)
        else {
            return Err(SqliteMigrationError::UnknownRecordedVersion { version });
        };
        if planned_migration.digest != found_digest {
            return Err(SqliteMigrationError::DigestMismatch {
                version,
                expected: planned_migration.digest.clone(),
                found: found_digest,
            });
        }
    }
    Ok(())
}

fn calculate_migration_digest(sql: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(sql.as_bytes());
    format!("sha256:{:x}", digest.finalize())
}

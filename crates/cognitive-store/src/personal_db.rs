//! Personal dual-database preparation on the XDG layout (P1-T01).
//!
//! Ensures the authority and installation SQLite files exist under the
//! resolved layout, applies the versioned migration plans through the
//! fail-closed adapter from P0-T04, and refuses concurrent migration via a
//! runtime lock file. Cross-database atomicity is intentionally not claimed.

use crate::installation::INSTALLATION_SCHEMA_V1;
use crate::layout::{PersonalDataLayout, PersonalLayoutError, restrict_private_file};
use crate::migration::{
    MigrationExecutionMode, MigrationExecutionReport, MigrationPlanEntry, SqliteMigrationError,
    execute_sqlite_migration_plan,
};
use crate::scheduler::{scheduler_binding_migration_entry, scheduler_migration_entry};
use crate::sqlite::AUTHORITY_SCHEMA_V1;
use crate::worker_authorization::{
    continuation_authority_migration_entry, daemon_authorization_snapshot_migration_entry,
    daemon_operation_descriptor_migration_entry,
    worker_authorization_lease_binding_migration_entry, worker_authorization_migration_entry,
    worker_iteration_authorization_consumption_migration_entry,
    worker_iteration_authorization_migration_entry,
};
use rusqlite::Connection;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Local report from preparing both Personal databases. Not release evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonalDatabasePrepareReport {
    authority: MigrationExecutionReport,
    installation: MigrationExecutionReport,
    authority_backup_path: Option<PathBuf>,
    installation_backup_path: Option<PathBuf>,
}

impl PersonalDatabasePrepareReport {
    /// Authority migration report for this prepare invocation.
    pub fn authority(&self) -> &MigrationExecutionReport {
        &self.authority
    }

    /// Installation migration report for this prepare invocation.
    pub fn installation(&self) -> &MigrationExecutionReport {
        &self.installation
    }

    /// Pre-migration authority backup path when an apply wrote one.
    pub fn authority_backup_path(&self) -> Option<&Path> {
        self.authority_backup_path.as_deref()
    }

    /// Pre-migration installation backup path when an apply wrote one.
    pub fn installation_backup_path(&self) -> Option<&Path> {
        self.installation_backup_path.as_deref()
    }
}

/// Production authority migration plan: v1 = full base schema, v2 = durable
/// scheduler persistence, v3 = immutable scheduler TaskBinding identity, v4
/// = immutable operation candidate proposal persistence, v5 = daemon-only
/// immutable operation descriptor registry, v6 = daemon authorization
/// snapshots, v7 = immutable worker iteration authorization storage, v8 =
/// immutable worker authorization consumption records, and v9 = immutable
/// WIA-to-scheduler lease binding records.
pub fn authority_migration_plan() -> Vec<MigrationPlanEntry> {
    vec![
        MigrationPlanEntry::new(1, AUTHORITY_SCHEMA_V1),
        scheduler_migration_entry(),
        scheduler_binding_migration_entry(),
        worker_authorization_migration_entry(),
        daemon_operation_descriptor_migration_entry(),
        daemon_authorization_snapshot_migration_entry(),
        worker_iteration_authorization_migration_entry(),
        worker_iteration_authorization_consumption_migration_entry(),
        worker_authorization_lease_binding_migration_entry(),
        continuation_authority_migration_entry(),
    ]
}

/// Production installation migration plan (currently version 1 = full schema).
pub fn installation_migration_plan() -> Vec<MigrationPlanEntry> {
    vec![MigrationPlanEntry::new(1, INSTALLATION_SCHEMA_V1)]
}

/// Create layout directories, acquire the exclusive migration lock, ensure both
/// SQLite files exist, and apply the production migration plans.
///
/// Failure leaves each database at its last committed schema (per-database
/// transactional rollback). A successful authority apply followed by an
/// installation failure is reported as an error; the authority backup path
/// remains on disk for operator recovery. This is not cross-DB atomicity.
pub fn prepare_personal_databases(
    layout: &PersonalDataLayout,
) -> Result<PersonalDatabasePrepareReport, PersonalLayoutError> {
    layout.ensure_directories()?;
    let _migration_lock = acquire_migration_lock(layout)?;

    let authority_plan = authority_migration_plan();
    let installation_plan = installation_migration_plan();

    ensure_sqlite_database_file(&layout.authority_database_path())?;
    ensure_sqlite_database_file(&layout.installation_database_path())?;

    let authority_backup_path = unique_backup_path(layout, "authority")?;
    let authority_report = execute_sqlite_migration_plan(
        &layout.authority_database_path(),
        &authority_plan,
        MigrationExecutionMode::Apply {
            backup_database_path: authority_backup_path.clone(),
        },
    )
    .map_err(map_migration_error)?;
    restrict_private_file(&layout.authority_database_path())?;
    if authority_backup_path.exists() {
        restrict_private_file(&authority_backup_path)?;
    }

    let installation_backup_path = unique_backup_path(layout, "installations")?;
    let installation_report = match execute_sqlite_migration_plan(
        &layout.installation_database_path(),
        &installation_plan,
        MigrationExecutionMode::Apply {
            backup_database_path: installation_backup_path.clone(),
        },
    ) {
        Ok(report) => report,
        Err(error) => {
            return Err(PersonalLayoutError::DatabasePreparation {
                detail: format!(
                    "installation migration failed after authority prepare; \
                     authority backup={}: {error}",
                    authority_backup_path.display()
                ),
            });
        }
    };
    restrict_private_file(&layout.installation_database_path())?;
    if installation_backup_path.exists() {
        restrict_private_file(&installation_backup_path)?;
    }

    Ok(PersonalDatabasePrepareReport {
        authority: authority_report,
        installation: installation_report,
        authority_backup_path: Some(authority_backup_path),
        installation_backup_path: Some(installation_backup_path),
    })
}

/// Apply an explicit plan to one database under the layout backup policy.
///
/// Intended for focused upgrade tests and future operator tools. Production
/// callers should prefer [`prepare_personal_databases`].
pub fn apply_database_migration_plan(
    database_path: &Path,
    backup_database_path: &Path,
    migration_plan: &[MigrationPlanEntry],
) -> Result<MigrationExecutionReport, PersonalLayoutError> {
    ensure_sqlite_database_file(database_path)?;
    let report = execute_sqlite_migration_plan(
        database_path,
        migration_plan,
        MigrationExecutionMode::Apply {
            backup_database_path: backup_database_path.to_path_buf(),
        },
    )
    .map_err(map_migration_error)?;
    restrict_private_file(database_path)?;
    if backup_database_path.exists() {
        restrict_private_file(backup_database_path)?;
    }
    Ok(report)
}

fn ensure_sqlite_database_file(path: &Path) -> Result<(), PersonalLayoutError> {
    if path.exists() {
        restrict_private_file(path)?;
        return Ok(());
    }
    if let Some(parent_directory) = path.parent() {
        crate::layout::create_private_directory(parent_directory)?;
    }
    let connection = Connection::open(path).map_err(|source| PersonalLayoutError::Filesystem {
        operation: "create empty sqlite database",
        source: std::io::Error::other(source.to_string()),
    })?;
    connection
        .execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|source| PersonalLayoutError::Filesystem {
            operation: "initialize empty sqlite database",
            source: std::io::Error::other(source.to_string()),
        })?;
    drop(connection);
    restrict_private_file(path)?;
    Ok(())
}

fn unique_backup_path(
    layout: &PersonalDataLayout,
    database_stem: &str,
) -> Result<PathBuf, PersonalLayoutError> {
    let timestamp_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    for attempt in 0..32u32 {
        let candidate = layout.backups_dir().join(format!(
            "{database_stem}.before-migration.{timestamp_seconds}-{attempt}.sqlite"
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(PersonalLayoutError::DatabasePreparation {
        detail: format!("unable to allocate unique backup path for {database_stem}"),
    })
}

struct MigrationLockGuard {
    lock_path: PathBuf,
}

impl Drop for MigrationLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

fn acquire_migration_lock(
    layout: &PersonalDataLayout,
) -> Result<MigrationLockGuard, PersonalLayoutError> {
    let lock_path = layout.migration_lock_path();
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
    {
        Ok(file) => {
            write_lock_payload(&file)?;
            restrict_private_file(&lock_path)?;
            Ok(MigrationLockGuard { lock_path })
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(PersonalLayoutError::LayoutLocked {
                detail: format!(
                    "migration.lock already exists at {}; refuse concurrent layout mutation",
                    lock_path.display()
                ),
            })
        }
        Err(source) => Err(PersonalLayoutError::Filesystem {
            operation: "create migration lock",
            source,
        }),
    }
}

fn write_lock_payload(file: &File) -> Result<(), PersonalLayoutError> {
    use std::io::Write;
    let payload = format!("pid={} purpose=personal-migration\n", std::process::id());
    let mut lock_file = file;
    lock_file
        .write_all(payload.as_bytes())
        .map_err(|source| PersonalLayoutError::Filesystem {
            operation: "write migration lock payload",
            source,
        })
}

fn map_migration_error(error: SqliteMigrationError) -> PersonalLayoutError {
    match error {
        SqliteMigrationError::UnsafeCopyDestination { detail } => {
            PersonalLayoutError::DatabasePreparation {
                detail: format!("backup/copy destination rejected: {detail}"),
            }
        }
        SqliteMigrationError::DigestMismatch {
            version,
            expected,
            found,
        } => PersonalLayoutError::DatabasePreparation {
            detail: format!(
                "migration digest mismatch at version {version}: expected {expected}, found {found}"
            ),
        },
        SqliteMigrationError::UnknownRecordedVersion { version } => {
            PersonalLayoutError::DatabasePreparation {
                detail: format!("database records unknown migration version {version}"),
            }
        }
        SqliteMigrationError::InvalidPlan { detail } => PersonalLayoutError::DatabasePreparation {
            detail: format!("invalid migration plan: {detail}"),
        },
        SqliteMigrationError::Sqlite { operation, source } => {
            PersonalLayoutError::DatabasePreparation {
                detail: format!("sqlite {operation}: {source}"),
            }
        }
    }
}

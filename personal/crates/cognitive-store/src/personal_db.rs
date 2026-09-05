//! Personal dual-database preparation on the XDG layout (P1-T01).
//!
//! Ensures the authority and installation SQLite files exist under the
//! resolved layout, applies the versioned migration plans through the
//! fail-closed adapter from P0-T04, and refuses concurrent migration via a
//! runtime lock file. Cross-database atomicity is intentionally not claimed.

use crate::attempt_artifacts::attempt_artifact_migration_entry;
use crate::context_store::{
    context_authorization_fact_migration_entry, context_store_migration_entry,
    scheduler_execution_policy_migration_entry, workspace_context_source_migration_entry,
};
use crate::conversation::conversation_migration_entry;
use crate::employee::employee_migration_entry;
use crate::hosted_dsh::hosted_dsh_migration_entry;
use crate::hosted_dsh_attempt::hosted_dsh_attempt_migration_entry;
use crate::installation::{
    INSTALLATION_SCHEMA_V1, INSTALLATION_SCHEMA_V2, INSTALLATION_SCHEMA_V3, INSTALLATION_SCHEMA_V4,
};
use crate::layout::{PersonalDataLayout, PersonalLayoutError, restrict_private_file};
use crate::memory_skill_consumption::memory_skill_consumption_migration_entry;
use crate::memory_store::{
    memory_admission_migration_entry, memory_expiry_migration_entry,
    memory_lifecycle_migration_entry, memory_search_migration_entry,
    memory_version_migration_entry,
};
use crate::migration::{
    MigrationExecutionMode, MigrationExecutionReport, MigrationPlanEntry, SqliteMigrationError,
    execute_sqlite_migration_plan,
};
use crate::project_aggregate::{
    approval_preview_narrow_migration_entry, project_aggregate_migration_entry,
    standing_approval_policy_migration_entry,
};
use crate::project_chat::project_chat_migration_entry;
use crate::provider_control_plane::provider_control_plane_migration_entry;
use crate::routine::routine_migration_entry;
use crate::routine_arming::routine_arming_migration_entry;
use crate::scheduler::{scheduler_binding_migration_entry, scheduler_migration_entry};
use crate::skill_store::{
    skill_binding_revocation_migration_entry, skill_package_migration_entry,
    skill_revision_lineage_migration_entry,
};
use crate::sqlite::AUTHORITY_SCHEMA_V1;
use crate::vault::vault_migration_entry;
use crate::windows_host::windows_host_migration_entry;
use crate::worker_authorization::{
    continuation_authority_consumption_migration_entry, continuation_authority_migration_entry,
    daemon_authorization_snapshot_migration_entry, daemon_operation_descriptor_migration_entry,
    worker_authorization_lease_binding_migration_entry, worker_authorization_migration_entry,
    worker_iteration_authorization_consumption_migration_entry,
    worker_iteration_authorization_migration_entry,
};
use crate::x_connector::x_connector_migration_entry;
use rusqlite::Connection;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// P13-T11 reflection / Member Runtime improvement. Nested here so the
/// module is public without editing `lib.rs` (sibling P13-T10 owns that file).
#[path = "reflection.rs"]
pub mod reflection;

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
/// WIA-to-scheduler lease binding records, v10 = private verified-continuation
/// evidence, v11 = private continuation-to-scheduler handoff bindings, and
/// v12 = durable immutable Context request/view records, and v13 =
/// daemon-admitted append-only workspace Context sources, and v24 =
/// append-only Memory/Skill consumption records keyed by Task/epoch/request/session,
/// and v25 = Provider Control Plane accounts, catalog, bindings, usage, budgets,
/// alerts, and redacted audit facts (no secret columns), and v26 = Personal-private
/// Project aggregate (Project / CharterRevision / PlanRevision / Stage / Gap /
/// Draft / Candidate / ApprovalPreview / StageTestFact / AcceptanceFact), and
/// v27 = Role Blueprint / Assignment / Employee / Grant (P11-T04), and
/// v28 = Personal-private conversation archive (P11-T05; new identifier, not
/// a reinterpretation of `conversation-projection/0.1`), and
/// v29 = ApprovalPreview `superseded_by` for HITL narrow (P11-T09), and
/// v30 = grant-expansion subject_kind plus StandingApprovalPolicy time-box
/// (`expires_at` required, ≤7d; Settings list/revoke), and
/// v31 = hidden hosted DSH managed child (`p11_hosted_dsh_child`; P11-T07), and
/// v32 = Markdown Vault documents / rebuildable index / conflicts (P11-T10),
/// and v33 = Routine revision / Trigger occurrence ledger (P11-T08; reuses
/// `scheduler_entries`, no second scheduler), and
/// v34 = Windows host Personal Home / lifecycle / missed / ordered recovery
/// (P11-T02; not a second credential plane; native E2E remains not-run), and
/// v35 = X/Twitter connector account / preview / publish ledger (P11-T14;
/// live X API remains not-run; not a P0 hero path), and
/// v36 = hosted DSH artifact health/update/rollback facts plus the
/// persist-before-dispatch Attempt / frame ledger (P13-T02; `completion_claimed`
/// CHECK=0, `verification_status` CHECK='not-run'; no `success` terminal), and
/// v37 = Attempt artifacts in the daemon CAS (`p13_attempt_artifact`),
/// independent verifier evidence (`p13_artifact_evidence`, append-only,
/// verifier-identity CHECK), last-ring run acceptance (`p13_run_acceptance`,
/// last-ring CHECK), external-send Intents (`p13_external_send`, `published`
/// CHECK=0) plus the `run-acceptance` / `external-send` ApprovalPreview
/// subject kinds (P13-T04; table rebuild, v30 precedent), and
/// v38 = Routine arming after G2 plus occurrence dispatch / outcome columns
/// (P13-T05; `p11_routine_occurrence` rebuilt to admit `attempted`;
/// `completion_claimed` CHECK=0; outcomes are daemon-observed Attempt
/// terminals, never `success`; the daemon scheduler tick is the only
/// dispatcher of `task://personal/routine/*` rows), and
/// v39 = Project group chat Owner turns (`p13_project_chat_turn`; mention /
/// routing / candidate envelope; chat never applies a PlanRevision; canvas
/// Confirm is the only writer; secret-shaped body refused), and
/// v40 = daemon-generated reflection candidates (`p13_reflection_candidate`)
/// plus versioned Member Runtime improvement (`p13_runtime_improvement`) and
/// cross-Project Role Template proposals (`p13_role_template_proposal`)
/// (P13-T11; `completion_claimed` CHECK=0; `model_self_report` CHECK=0;
/// `implicit_blueprint` CHECK=0; `silent_reuse` CHECK=0; Owner preview
/// required; rebuilds `p11_approval_preview` for `member-runtime-revision`
/// and `role-template-proposal`).
/// P11-T12 honest usage is a labelled read of v25 usage/bindings (no new
/// migration): unknown cost never serializes as 0; Project/employee/Task
/// Provider bindings are explicit unbound.
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
        continuation_authority_consumption_migration_entry(),
        context_store_migration_entry(),
        workspace_context_source_migration_entry(),
        context_authorization_fact_migration_entry(),
        scheduler_execution_policy_migration_entry(),
        memory_admission_migration_entry(),
        memory_search_migration_entry(),
        memory_lifecycle_migration_entry(),
        memory_expiry_migration_entry(),
        memory_version_migration_entry(),
        skill_package_migration_entry(),
        skill_binding_revocation_migration_entry(),
        skill_revision_lineage_migration_entry(),
        memory_skill_consumption_migration_entry(),
        provider_control_plane_migration_entry(),
        project_aggregate_migration_entry(),
        employee_migration_entry(),
        conversation_migration_entry(),
        approval_preview_narrow_migration_entry(),
        standing_approval_policy_migration_entry(),
        hosted_dsh_migration_entry(),
        vault_migration_entry(),
        routine_migration_entry(),
        windows_host_migration_entry(),
        x_connector_migration_entry(),
        hosted_dsh_attempt_migration_entry(),
        attempt_artifact_migration_entry(),
        routine_arming_migration_entry(),
        project_chat_migration_entry(),
        reflection::reflection_migration_entry(),
    ]
}

/// Production installation migration plan: v1 = package/root/quarantine schema,
/// v2 = Agent registration/instance identity, v3 = SidecarSession identity,
/// v4 = fenced SidecarSession process-attempt binding.
pub fn installation_migration_plan() -> Vec<MigrationPlanEntry> {
    vec![
        MigrationPlanEntry::new(1, INSTALLATION_SCHEMA_V1),
        MigrationPlanEntry::new(2, INSTALLATION_SCHEMA_V2),
        MigrationPlanEntry::new(3, INSTALLATION_SCHEMA_V3),
        MigrationPlanEntry::new(4, INSTALLATION_SCHEMA_V4),
    ]
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

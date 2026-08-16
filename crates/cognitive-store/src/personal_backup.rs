//! Personal user backup inventory and Memory/Skill export planning (P7-T02).
//!
//! D01 plans which layout paths may enter a user-facing backup archive.
//! Memory, Skill, bindings, state, config, and artifact roots are eligible.
//! Secret Store material, bootstrap secrets, and provider opaque refs stay
//! excluded.
//!
//! D02 builds a digest-bound Memory/Skill/bindings export plan that may only
//! reference D01-approved inventory categories and never carries secret
//! material.
//!
//! D03 runs restore preflight and migration compatibility checks that reject
//! incompatible or incomplete backups before any mutation.
//!
//! D04 plans transactional update/rollback/uninstall over D01–D03 evidence.
//! Secret material stays excluded from destructive uninstall by default.
//!
//! P2-T27 adds public archive I/O: a digest-bound directory archive, restore
//! preflight, and a transactional live overlay that never copies raw authority
//! SQLite, secrets, bearer material, or provider-config. This module does not
//! claim Gate/release outcomes.

use crate::layout::PersonalDataLayout;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Public archive schema identity for `cognitive backup` / `restore`.
pub const BACKUP_ARCHIVE_SCHEMA: &str = "cognitiveos.personal.backup/0.1";
/// Archive format version consumed by restore preflight.
pub const BACKUP_ARCHIVE_SCHEMA_VERSION: u32 = 1;

const FORBIDDEN_BACKUP_MARKERS: &[&str] = &[
    "secret",
    "credential",
    "api_key",
    "api-key",
    "private_key",
    "ssv1:",
    "sk-",
    "local-bootstrap.secret",
    "provider-config.json",
];

const MEMORY_SKILL_EXPORT_CATEGORY: &str = "authority-db";

/// One included backup root with a stable category label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupInventoryEntry {
    pub category: &'static str,
    pub path: PathBuf,
}

/// Secret-excluding backup inventory over a Personal layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalBackupInventory {
    pub entries: Vec<BackupInventoryEntry>,
    pub excluded_secret_paths: Vec<PathBuf>,
}

/// Kind of Memory/Skill/bindings export unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupExportKind {
    Memory,
    SkillPackage,
    SkillRevision,
    SkillBinding,
}

/// One digest-bound Memory/Skill/bindings export unit.
///
/// Units must reference a D01-approved inventory category (always
/// `authority-db` for Memory/Skill/bindings facts) and carry non-secret
/// digests only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupExportUnit {
    pub kind: BackupExportKind,
    pub resource_id: String,
    pub inventory_category: &'static str,
    /// Primary content digest (Memory source/candidate, Skill content, or
    /// binding revision content digest).
    pub content_digest: String,
    /// Secondary digest when required (Memory candidate digest, Skill
    /// manifest digest for packages/bindings).
    pub related_digest: Option<String>,
    /// Required for [`BackupExportKind::SkillBinding`]: pinned revision id.
    pub binding_revision_id: Option<String>,
}

/// Digest-bound Memory/Skill/bindings export plan over a D01 inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalBackupExportPlan {
    pub inventory_categories: Vec<&'static str>,
    pub units: Vec<BackupExportUnit>,
    pub plan_digest: String,
}

/// Caller-described backup archive facts for restore preflight (D03).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupRestoreCandidate {
    pub export_plan: PersonalBackupExportPlan,
    pub categories_present: Vec<&'static str>,
    pub backup_schema_version: u32,
    pub expected_schema_version: u32,
    pub migration_plan_digest: String,
}

/// Successful restore preflight result; no mutation has occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalBackupRestorePreflight {
    pub export_plan_digest: String,
    pub backup_schema_version: u32,
    pub migration_plan_digest: String,
}

/// Transactional lifecycle operation planned over backup evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonalLifecycleOperation {
    Update,
    Rollback,
    Uninstall,
}

/// Uninstall target classes. Secret is never selected by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UninstallTargetClass {
    Binary,
    Config,
    Cache,
    Data,
    Secret,
}

/// Planned transactional lifecycle step. Commit is a separate explicit action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalLifecyclePlan {
    pub operation: PersonalLifecycleOperation,
    pub preflight: PersonalBackupRestorePreflight,
    pub uninstall_targets: Vec<UninstallTargetClass>,
    pub staged: bool,
}

/// Fail-closed errors for backup inventory and export planning.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PersonalBackupError {
    #[error("backup inventory proposal includes a forbidden secret path")]
    SecretPathIncluded,
    #[error("backup inventory proposal is missing a required category: {0}")]
    MissingRequiredCategory(&'static str),
    #[error("backup export unit references an unapproved inventory category")]
    UnapprovedInventoryCategory,
    #[error("backup export payload includes secret contamination")]
    SecretContamination,
    #[error("backup export binding is missing required revision or digest material")]
    MissingBinding,
    #[error("backup export unit is missing a required content digest")]
    MissingDigest,
    #[error("backup restore preflight rejected incompatible schema versions")]
    SchemaIncompatible,
    #[error("backup restore preflight found an incomplete backup archive")]
    IncompleteBackup,
    #[error("backup restore preflight rejected migration plan mismatch")]
    MigrationPreflightFailed,
    #[error("backup restore preflight rejected export plan digest mismatch")]
    ExportPlanDigestMismatch,
    #[error("lifecycle plan requires a successful restore preflight")]
    LifecyclePreflightRequired,
    #[error("lifecycle uninstall refused secret or unconfirmed data deletion")]
    UninstallConfirmationRequired,
    #[error("lifecycle commit refused because the plan was not staged")]
    LifecycleNotStaged,
    #[error("backup archive I/O failed: {0}")]
    ArchiveIo(String),
    #[error("backup archive part digest does not match the manifest")]
    ArchiveTampered,
    #[error("backup archive includes a forbidden secret path")]
    ArchiveSecretIncluded,
    #[error("backup archive tried to copy a raw authority SQLite file")]
    RawSqliteCopyForbidden,
    #[error("restore refused because it would leave partial live state")]
    RestorePartialRefused,
    #[error("backup archive manifest is missing or invalid")]
    ArchiveManifestInvalid,
    #[error("restore refused while the Personal daemon lock is present")]
    DaemonLockHeld,
    #[error("backup output path is inside the Personal layout")]
    ArchiveOutputInsideLayout,
    #[error("backup output path already exists")]
    ArchiveOutputExists,
}

/// Plan the default user-facing backup inventory for a Personal layout.
///
/// Always excludes the bootstrap secret path and any caller-supplied secret
/// roots. Rejects proposals that reintroduce forbidden secret markers.
pub fn plan_personal_backup_inventory(
    layout: &PersonalDataLayout,
    additional_secret_paths: &[PathBuf],
) -> Result<PersonalBackupInventory, PersonalBackupError> {
    let mut excluded_secret_paths = vec![layout.local_bootstrap_secret_path()];
    excluded_secret_paths.extend(additional_secret_paths.iter().cloned());
    excluded_secret_paths.push(layout.config_dir().join("provider-config.json"));

    let entries = vec![
        BackupInventoryEntry {
            category: "authority-db",
            path: layout.authority_database_path(),
        },
        BackupInventoryEntry {
            category: "installation-db",
            path: layout.installation_database_path(),
        },
        BackupInventoryEntry {
            category: "config",
            path: layout.config_dir().to_path_buf(),
        },
        BackupInventoryEntry {
            category: "data",
            path: layout.data_dir().to_path_buf(),
        },
        BackupInventoryEntry {
            category: "state",
            path: layout.state_dir().to_path_buf(),
        },
        BackupInventoryEntry {
            category: "artifacts",
            path: layout.data_dir().join("artifacts"),
        },
    ];

    validate_backup_inventory(&entries, &excluded_secret_paths)?;
    Ok(PersonalBackupInventory {
        entries,
        excluded_secret_paths,
    })
}

/// Validate a caller-built inventory against secret-exclusion rules.
pub fn validate_backup_inventory(
    entries: &[BackupInventoryEntry],
    excluded_secret_paths: &[PathBuf],
) -> Result<(), PersonalBackupError> {
    let required = [
        "authority-db",
        "installation-db",
        "config",
        "data",
        "state",
        "artifacts",
    ];
    for category in required {
        if !entries.iter().any(|entry| entry.category == category) {
            return Err(PersonalBackupError::MissingRequiredCategory(category));
        }
    }

    for entry in entries {
        if inventory_path_is_forbidden(&entry.path, excluded_secret_paths) {
            return Err(PersonalBackupError::SecretPathIncluded);
        }
    }
    Ok(())
}

/// Plan a digest-bound Memory/Skill/bindings export over a D01 inventory.
///
/// Consumes only D01-approved categories. Memory/Skill/bindings facts must
/// target `authority-db`. Rejects secret contamination and incomplete
/// Skill bindings before any archive write.
pub fn plan_memory_skill_export(
    inventory: &PersonalBackupInventory,
    units: &[BackupExportUnit],
) -> Result<PersonalBackupExportPlan, PersonalBackupError> {
    validate_backup_inventory(&inventory.entries, &inventory.excluded_secret_paths)?;

    let approved: Vec<&'static str> = inventory
        .entries
        .iter()
        .map(|entry| entry.category)
        .collect();
    let mut planned = Vec::with_capacity(units.len());

    for unit in units {
        if !approved.contains(&unit.inventory_category) {
            return Err(PersonalBackupError::UnapprovedInventoryCategory);
        }
        if unit.inventory_category != MEMORY_SKILL_EXPORT_CATEGORY {
            return Err(PersonalBackupError::UnapprovedInventoryCategory);
        }
        validate_export_unit(unit)?;
        planned.push(unit.clone());
    }

    let plan_digest = bind_export_plan_digest(&planned);
    Ok(PersonalBackupExportPlan {
        inventory_categories: approved,
        units: planned,
        plan_digest,
    })
}

/// Preflight a restore against D01 inventory completeness, D02 export digests,
/// and schema/migration compatibility. Rejects before any mutation.
pub fn preflight_personal_backup_restore(
    inventory: &PersonalBackupInventory,
    candidate: &BackupRestoreCandidate,
) -> Result<PersonalBackupRestorePreflight, PersonalBackupError> {
    validate_backup_inventory(&inventory.entries, &inventory.excluded_secret_paths)?;

    if candidate.backup_schema_version == 0
        || candidate.expected_schema_version == 0
        || candidate.backup_schema_version != candidate.expected_schema_version
    {
        return Err(PersonalBackupError::SchemaIncompatible);
    }

    if candidate.migration_plan_digest.trim().is_empty()
        || export_text_is_contaminated(&candidate.migration_plan_digest)
    {
        return Err(PersonalBackupError::MigrationPreflightFailed);
    }

    let required = [
        "authority-db",
        "installation-db",
        "config",
        "data",
        "state",
        "artifacts",
    ];
    for category in required {
        if !candidate.categories_present.contains(&category) {
            return Err(PersonalBackupError::IncompleteBackup);
        }
        if !inventory
            .entries
            .iter()
            .any(|entry| entry.category == category)
        {
            return Err(PersonalBackupError::IncompleteBackup);
        }
    }

    let recomputed = bind_export_plan_digest(&candidate.export_plan.units);
    if recomputed != candidate.export_plan.plan_digest {
        return Err(PersonalBackupError::ExportPlanDigestMismatch);
    }

    // Re-validate units against inventory to catch tampered plans.
    let _ = plan_memory_skill_export(inventory, &candidate.export_plan.units)?;

    Ok(PersonalBackupRestorePreflight {
        export_plan_digest: candidate.export_plan.plan_digest.clone(),
        backup_schema_version: candidate.backup_schema_version,
        migration_plan_digest: candidate.migration_plan_digest.clone(),
    })
}

/// Plan a transactional update/rollback/uninstall over a successful D03
/// preflight. Destructive data uninstall requires explicit confirmation.
/// Secret targets are always refused.
pub fn plan_personal_lifecycle(
    preflight: &PersonalBackupRestorePreflight,
    operation: PersonalLifecycleOperation,
    uninstall_targets: &[UninstallTargetClass],
    confirm_data_deletion: bool,
) -> Result<PersonalLifecyclePlan, PersonalBackupError> {
    if preflight.export_plan_digest.trim().is_empty()
        || preflight.migration_plan_digest.trim().is_empty()
        || preflight.backup_schema_version == 0
    {
        return Err(PersonalBackupError::LifecyclePreflightRequired);
    }

    let mut targets = Vec::new();
    match operation {
        PersonalLifecycleOperation::Update | PersonalLifecycleOperation::Rollback => {
            if !uninstall_targets.is_empty() {
                return Err(PersonalBackupError::UninstallConfirmationRequired);
            }
        }
        PersonalLifecycleOperation::Uninstall => {
            if uninstall_targets.is_empty() {
                return Err(PersonalBackupError::UninstallConfirmationRequired);
            }
            for target in uninstall_targets {
                match target {
                    UninstallTargetClass::Secret => {
                        return Err(PersonalBackupError::UninstallConfirmationRequired);
                    }
                    UninstallTargetClass::Data if !confirm_data_deletion => {
                        return Err(PersonalBackupError::UninstallConfirmationRequired);
                    }
                    UninstallTargetClass::Binary
                    | UninstallTargetClass::Config
                    | UninstallTargetClass::Cache
                    | UninstallTargetClass::Data => targets.push(*target),
                }
            }
        }
    }

    Ok(PersonalLifecyclePlan {
        operation,
        preflight: preflight.clone(),
        uninstall_targets: targets,
        staged: true,
    })
}

/// Commit a previously staged lifecycle plan. This authority-path commit does
/// not delete host files; callers must still perform OS mutations separately.
pub fn commit_personal_lifecycle(
    plan: &PersonalLifecyclePlan,
) -> Result<PersonalLifecyclePlan, PersonalBackupError> {
    if !plan.staged {
        return Err(PersonalBackupError::LifecycleNotStaged);
    }
    Ok(PersonalLifecyclePlan {
        staged: false,
        ..plan.clone()
    })
}

/// Abort a staged lifecycle plan without committing destructive work.
pub fn abort_personal_lifecycle(
    plan: &PersonalLifecyclePlan,
) -> Result<PersonalLifecyclePlan, PersonalBackupError> {
    if !plan.staged {
        return Err(PersonalBackupError::LifecycleNotStaged);
    }
    Ok(PersonalLifecyclePlan {
        staged: false,
        uninstall_targets: Vec::new(),
        ..plan.clone()
    })
}

/// Redacted receipt for a daemon-written backup archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupArchiveReceipt {
    pub schema: String,
    pub archive_id: String,
    pub archive_path: PathBuf,
    pub manifest_digest: String,
    pub export_plan_digest: String,
    pub sqlite_copied: bool,
    pub excluded_secret_count: usize,
}

/// Redacted receipt after a transactional restore of one archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupRestoreReceipt {
    pub schema: String,
    pub archive_id: String,
    pub restored_path: PathBuf,
    pub manifest_digest: String,
    pub live_applied: bool,
}

/// Options for [`restore_personal_backup_archive_with_options`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackupRestoreOptions {
    /// Overlay verified archive parts onto the live layout after staging.
    pub apply_live: bool,
    /// Test-only: abort after snapshot and before live overlay.
    pub inject_fault_before_apply: bool,
    /// When true, refuse restore if `daemon.lock` is present.
    pub refuse_if_daemon_lock: bool,
}

impl Default for BackupRestoreOptions {
    fn default() -> Self {
        Self {
            apply_live: true,
            inject_fault_before_apply: false,
            refuse_if_daemon_lock: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BackupArchiveManifest {
    schema: String,
    backup_schema_version: u32,
    migration_plan_digest: String,
    export_plan_digest: String,
    categories: Vec<String>,
    parts: Vec<BackupArchivePart>,
    excluded_secret_names: Vec<String>,
    sqlite_copied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BackupArchivePart {
    path: String,
    sha256: String,
}

/// Write a secret-excluding, non-SQLite backup archive under `backups_dir`.
pub fn write_personal_backup_archive(
    layout: &PersonalDataLayout,
) -> Result<BackupArchiveReceipt, PersonalBackupError> {
    write_personal_backup_archive_to(layout, None)
}

/// Write a secret-excluding archive to `output_dir` or the default backups root.
pub fn write_personal_backup_archive_to(
    layout: &PersonalDataLayout,
    output_dir: Option<&Path>,
) -> Result<BackupArchiveReceipt, PersonalBackupError> {
    layout
        .ensure_directories()
        .map_err(|error| PersonalBackupError::ArchiveIo(error.to_string()))?;
    if let Some(output) = output_dir {
        if path_is_inside_layout(layout, output) {
            return Err(PersonalBackupError::ArchiveOutputInsideLayout);
        }
        if output.exists() {
            return Err(PersonalBackupError::ArchiveOutputExists);
        }
    }
    let inventory = plan_personal_backup_inventory(layout, &[])?;
    let export_plan = plan_memory_skill_export(&inventory, &[])?;
    let migration_plan_digest = bind_text_digest(BACKUP_ARCHIVE_SCHEMA);

    let staging = layout
        .backups_dir()
        .join(format!(".staging-{}", std::process::id()));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(archive_io)?;
    }
    fs::create_dir_all(staging.join("parts")).map_err(archive_io)?;

    let mut parts = Vec::new();
    for entry in &inventory.entries {
        let category_dir = staging.join("parts").join(entry.category);
        fs::create_dir_all(&category_dir).map_err(archive_io)?;
        if entry.category == "authority-db" {
            let export_path = category_dir.join("export.json");
            write_json_file(
                &export_path,
                &serde_json::json!({
                    "inventory_categories": export_plan.inventory_categories,
                    "plan_digest": export_plan.plan_digest,
                    "units": [],
                    "sqlite_copied": false
                }),
            )?;
            parts.push(part_record(&staging, &export_path)?);
            continue;
        }
        if entry.category == "installation-db" {
            let meta_path = category_dir.join("metadata.json");
            write_json_file(
                &meta_path,
                &serde_json::json!({
                    "sqlite_copied": false,
                    "migration_plan_digest": migration_plan_digest
                }),
            )?;
            parts.push(part_record(&staging, &meta_path)?);
            continue;
        }
        copy_tree_excluding_secrets_and_sqlite(
            &entry.path,
            &category_dir,
            &inventory.excluded_secret_paths,
            &mut parts,
            &staging,
        )?;
        if parts
            .iter()
            .all(|part| !part.path.starts_with(&format!("parts/{}", entry.category)))
        {
            let marker = category_dir.join("empty.json");
            write_json_file(&marker, &serde_json::json!({ "empty": true }))?;
            parts.push(part_record(&staging, &marker)?);
        }
    }

    let manifest = BackupArchiveManifest {
        schema: BACKUP_ARCHIVE_SCHEMA.to_owned(),
        backup_schema_version: BACKUP_ARCHIVE_SCHEMA_VERSION,
        migration_plan_digest: migration_plan_digest.clone(),
        export_plan_digest: export_plan.plan_digest.clone(),
        categories: inventory
            .entries
            .iter()
            .map(|entry| entry.category.to_owned())
            .collect(),
        parts,
        excluded_secret_names: inventory
            .excluded_secret_paths
            .iter()
            .filter_map(|path| {
                path.file_name()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .collect(),
        sqlite_copied: false,
    };
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| PersonalBackupError::ArchiveIo(error.to_string()))?;
    let manifest_digest = format!("{:x}", Sha256::digest(&manifest_bytes));
    fs::write(staging.join("manifest.json"), &manifest_bytes).map_err(archive_io)?;

    let archive_id = manifest_digest[..16].to_owned();
    let archive_path = match output_dir {
        Some(path) => path.to_path_buf(),
        None => layout.backups_dir().join("archives").join(&archive_id),
    };
    if archive_path.exists() {
        if output_dir.is_some() {
            let _ = fs::remove_dir_all(&staging);
            return Err(PersonalBackupError::ArchiveOutputExists);
        }
        fs::remove_dir_all(&archive_path).map_err(archive_io)?;
    }
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent).map_err(archive_io)?;
    }
    fs::rename(&staging, &archive_path).map_err(archive_io)?;

    Ok(BackupArchiveReceipt {
        schema: BACKUP_ARCHIVE_SCHEMA.to_owned(),
        archive_id,
        archive_path,
        manifest_digest,
        export_plan_digest: export_plan.plan_digest,
        sqlite_copied: false,
        excluded_secret_count: inventory.excluded_secret_paths.len(),
    })
}

/// Verify archive digests, schema, completeness, and secret/SQLite exclusion.
/// Does not mutate the live layout.
pub fn preflight_personal_backup_archive(
    layout: &PersonalDataLayout,
    archive_path: &Path,
) -> Result<PersonalBackupRestorePreflight, PersonalBackupError> {
    let manifest = load_and_verify_manifest(archive_path)?;
    if manifest.sqlite_copied {
        return Err(PersonalBackupError::RawSqliteCopyForbidden);
    }
    let inventory = plan_personal_backup_inventory(layout, &[])?;
    let export_plan = plan_memory_skill_export(&inventory, &[])?;
    if export_plan.plan_digest != manifest.export_plan_digest {
        return Err(PersonalBackupError::ExportPlanDigestMismatch);
    }
    let categories_present = [
        "authority-db",
        "installation-db",
        "config",
        "data",
        "state",
        "artifacts",
    ];
    for category in categories_present {
        if !manifest.categories.iter().any(|name| name == category) {
            return Err(PersonalBackupError::IncompleteBackup);
        }
    }
    let candidate = BackupRestoreCandidate {
        export_plan,
        categories_present: categories_present.to_vec(),
        backup_schema_version: manifest.backup_schema_version,
        expected_schema_version: BACKUP_ARCHIVE_SCHEMA_VERSION,
        migration_plan_digest: manifest.migration_plan_digest.clone(),
    };
    preflight_personal_backup_restore(&inventory, &candidate)
}

/// Restore one archive: stage a verified tree, then overlay live layout files.
///
/// Raw SQLite copies and secret-named files fail closed. A failure after
/// snapshot rolls the live trees back and returns
/// [`PersonalBackupError::RestorePartialRefused`].
pub fn restore_personal_backup_archive(
    layout: &PersonalDataLayout,
    archive_path: &Path,
) -> Result<BackupRestoreReceipt, PersonalBackupError> {
    restore_personal_backup_archive_with_options(
        layout,
        archive_path,
        BackupRestoreOptions::default(),
    )
}

/// Restore with explicit overlay / injected-fault options.
pub fn restore_personal_backup_archive_with_options(
    layout: &PersonalDataLayout,
    archive_path: &Path,
    options: BackupRestoreOptions,
) -> Result<BackupRestoreReceipt, PersonalBackupError> {
    layout
        .ensure_directories()
        .map_err(|error| PersonalBackupError::ArchiveIo(error.to_string()))?;
    let _preflight = preflight_personal_backup_archive(layout, archive_path)?;
    let manifest = load_and_verify_manifest(archive_path)?;
    if options.apply_live && options.refuse_if_daemon_lock && layout.daemon_lock_path().exists() {
        return Err(PersonalBackupError::DaemonLockHeld);
    }

    let staging = layout
        .runtime_dir()
        .join(format!("restore-staging-{}", std::process::id()));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(archive_io)?;
    }
    copy_verified_archive(archive_path, &staging, &manifest).inspect_err(|_| {
        let _ = fs::remove_dir_all(&staging);
    })?;

    let snapshot = layout
        .runtime_dir()
        .join(format!("restore-snapshot-{}", std::process::id()));
    if options.apply_live {
        if snapshot.exists() {
            fs::remove_dir_all(&snapshot).map_err(archive_io)?;
        }
        snapshot_live_trees(layout, &snapshot).inspect_err(|_| {
            let _ = fs::remove_dir_all(&staging);
            let _ = fs::remove_dir_all(&snapshot);
        })?;
        if options.inject_fault_before_apply {
            let _ = fs::remove_dir_all(&staging);
            let _ = fs::remove_dir_all(&snapshot);
            return Err(PersonalBackupError::RestorePartialRefused);
        }
        if let Err(error) = apply_verified_parts_to_live(layout, &staging) {
            let _ = restore_live_trees_from_snapshot(layout, &snapshot);
            let _ = fs::remove_dir_all(&staging);
            let _ = fs::remove_dir_all(&snapshot);
            return Err(error);
        }
        let _ = fs::remove_dir_all(&snapshot);
    }

    let restored_path = layout
        .backups_dir()
        .join("restored")
        .join(&manifest.export_plan_digest[..16.min(manifest.export_plan_digest.len())]);
    if let Some(parent) = restored_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            let _ = fs::remove_dir_all(&staging);
            archive_io(error)
        })?;
    }
    if restored_path.exists() {
        fs::remove_dir_all(&restored_path).map_err(|error| {
            let _ = fs::remove_dir_all(&staging);
            archive_io(error)
        })?;
    }
    fs::rename(&staging, &restored_path).map_err(|_| {
        let _ = fs::remove_dir_all(&staging);
        PersonalBackupError::RestorePartialRefused
    })?;

    Ok(BackupRestoreReceipt {
        schema: BACKUP_ARCHIVE_SCHEMA.to_owned(),
        archive_id: restored_path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default(),
        restored_path,
        manifest_digest: manifest_digest_of(archive_path)?,
        live_applied: options.apply_live,
    })
}

fn archive_io(error: io::Error) -> PersonalBackupError {
    PersonalBackupError::ArchiveIo(error.to_string())
}

fn path_is_inside_layout(layout: &PersonalDataLayout, path: &Path) -> bool {
    let roots = [
        layout.config_dir(),
        layout.data_dir(),
        layout.state_dir(),
        layout.cache_dir(),
        layout.runtime_dir(),
    ];
    roots.iter().any(|root| path.starts_with(root))
}

fn snapshot_live_trees(
    layout: &PersonalDataLayout,
    snapshot: &Path,
) -> Result<(), PersonalBackupError> {
    copy_tree_all(layout.config_dir(), &snapshot.join("config"))?;
    copy_tree_all(layout.data_dir(), &snapshot.join("data"))?;
    copy_tree_all(layout.state_dir(), &snapshot.join("state"))?;
    Ok(())
}

fn restore_live_trees_from_snapshot(
    layout: &PersonalDataLayout,
    snapshot: &Path,
) -> Result<(), PersonalBackupError> {
    replace_tree(&snapshot.join("config"), layout.config_dir())?;
    replace_tree(&snapshot.join("data"), layout.data_dir())?;
    replace_tree(&snapshot.join("state"), layout.state_dir())?;
    Ok(())
}

fn replace_tree(source: &Path, destination: &Path) -> Result<(), PersonalBackupError> {
    if destination.exists() {
        fs::remove_dir_all(destination).map_err(archive_io)?;
    }
    if source.exists() {
        copy_tree_all(source, destination)?;
    } else {
        fs::create_dir_all(destination).map_err(archive_io)?;
    }
    Ok(())
}

fn apply_verified_parts_to_live(
    layout: &PersonalDataLayout,
    staging: &Path,
) -> Result<(), PersonalBackupError> {
    overlay_tree(
        &staging.join("parts").join("config"),
        layout.config_dir(),
        layout,
    )?;
    overlay_tree(
        &staging.join("parts").join("data"),
        layout.data_dir(),
        layout,
    )?;
    overlay_tree(
        &staging.join("parts").join("state"),
        layout.state_dir(),
        layout,
    )?;
    overlay_tree(
        &staging.join("parts").join("artifacts"),
        &layout.data_dir().join("artifacts"),
        layout,
    )?;
    let export = staging
        .join("parts")
        .join("authority-db")
        .join("export.json");
    if export.exists() {
        let dest = layout.data_dir().join("memory-skill-export.json");
        fs::copy(&export, &dest).map_err(archive_io)?;
    }
    Ok(())
}

fn overlay_tree(
    source: &Path,
    destination: &Path,
    layout: &PersonalDataLayout,
) -> Result<(), PersonalBackupError> {
    if !source.exists() {
        return Ok(());
    }
    fs::create_dir_all(destination).map_err(archive_io)?;
    overlay_tree_inner(source, destination, layout)
}

fn overlay_tree_inner(
    source: &Path,
    destination: &Path,
    layout: &PersonalDataLayout,
) -> Result<(), PersonalBackupError> {
    for entry in fs::read_dir(source).map_err(archive_io)? {
        let entry = entry.map_err(archive_io)?;
        let from = entry.path();
        let name = entry.file_name();
        let name_lossy = name.to_string_lossy();
        if name_lossy == "empty.json" || name_lossy == "metadata.json" {
            continue;
        }
        let to = destination.join(&name);
        if from.is_dir() {
            fs::create_dir_all(&to).map_err(archive_io)?;
            overlay_tree_inner(&from, &to, layout)?;
        } else if !should_skip_live_file(&to, layout) {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent).map_err(archive_io)?;
            }
            fs::copy(&from, &to).map_err(archive_io)?;
        }
    }
    Ok(())
}

fn should_skip_live_file(path: &Path, layout: &PersonalDataLayout) -> bool {
    if inventory_path_is_forbidden(path, &layout_secret_exclusions(layout)) {
        return true;
    }
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    name.ends_with(".sqlite")
        || name.ends_with(".sqlite-wal")
        || name.ends_with(".sqlite-shm")
        || name == "daemon.lock"
        || name == "daemon.sock"
}

fn layout_secret_exclusions(layout: &PersonalDataLayout) -> Vec<PathBuf> {
    vec![
        layout.local_bootstrap_secret_path(),
        layout.config_dir().join("provider-config.json"),
    ]
}

fn copy_tree_all(source: &Path, destination: &Path) -> Result<(), PersonalBackupError> {
    if !source.exists() {
        return Ok(());
    }
    if source.is_file() {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(archive_io)?;
        }
        fs::copy(source, destination).map_err(archive_io)?;
        return Ok(());
    }
    fs::create_dir_all(destination).map_err(archive_io)?;
    for entry in fs::read_dir(source).map_err(archive_io)? {
        let entry = entry.map_err(archive_io)?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() {
            copy_tree_all(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(archive_io)?;
        }
    }
    Ok(())
}

fn bind_text_digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}

fn write_json_file(path: &Path, value: &serde_json::Value) -> Result<(), PersonalBackupError> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| PersonalBackupError::ArchiveIo(error.to_string()))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(archive_io)?;
    }
    let mut file = fs::File::create(path).map_err(archive_io)?;
    file.write_all(&bytes).map_err(archive_io)?;
    Ok(())
}

fn part_record(root: &Path, path: &Path) -> Result<BackupArchivePart, PersonalBackupError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| PersonalBackupError::ArchiveManifestInvalid)?;
    let bytes = fs::read(path).map_err(archive_io)?;
    Ok(BackupArchivePart {
        path: relative.to_string_lossy().replace('\\', "/"),
        sha256: format!("{:x}", Sha256::digest(&bytes)),
    })
}

fn copy_tree_excluding_secrets_and_sqlite(
    source: &Path,
    destination: &Path,
    excluded_secret_paths: &[PathBuf],
    parts: &mut Vec<BackupArchivePart>,
    archive_root: &Path,
) -> Result<(), PersonalBackupError> {
    if !source.exists() {
        return Ok(());
    }
    if source.is_file() {
        return copy_one_file(
            source,
            destination,
            excluded_secret_paths,
            parts,
            archive_root,
        );
    }
    let entries = fs::read_dir(source).map_err(archive_io)?;
    for entry in entries {
        let entry = entry.map_err(archive_io)?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() {
            let dir_name = entry.file_name();
            let dir_name = dir_name.to_string_lossy();
            if dir_name == "backups"
                || dir_name.starts_with(".staging-")
                || dir_name.starts_with("restore-staging-")
                || dir_name.starts_with("restore-snapshot-")
            {
                continue;
            }
            fs::create_dir_all(&to).map_err(archive_io)?;
            copy_tree_excluding_secrets_and_sqlite(
                &from,
                &to,
                excluded_secret_paths,
                parts,
                archive_root,
            )?;
        } else {
            copy_one_file(&from, &to, excluded_secret_paths, parts, archive_root)?;
        }
    }
    Ok(())
}

fn copy_one_file(
    source: &Path,
    destination: &Path,
    excluded_secret_paths: &[PathBuf],
    parts: &mut Vec<BackupArchivePart>,
    archive_root: &Path,
) -> Result<(), PersonalBackupError> {
    if inventory_path_is_forbidden(source, excluded_secret_paths) {
        return Ok(());
    }
    let name = source
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if name.ends_with(".sqlite") || name.ends_with(".sqlite-wal") || name.ends_with(".sqlite-shm") {
        return Ok(());
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(archive_io)?;
    }
    fs::copy(source, destination).map_err(archive_io)?;
    parts.push(part_record(archive_root, destination)?);
    Ok(())
}

fn load_and_verify_manifest(
    archive_path: &Path,
) -> Result<BackupArchiveManifest, PersonalBackupError> {
    let manifest_path = archive_path.join("manifest.json");
    let bytes =
        fs::read(&manifest_path).map_err(|_| PersonalBackupError::ArchiveManifestInvalid)?;
    let manifest: BackupArchiveManifest =
        serde_json::from_slice(&bytes).map_err(|_| PersonalBackupError::ArchiveManifestInvalid)?;
    if manifest.schema != BACKUP_ARCHIVE_SCHEMA {
        return Err(PersonalBackupError::SchemaIncompatible);
    }
    for part in &manifest.parts {
        let path = archive_path.join(&part.path);
        if inventory_path_is_forbidden(&path, &[])
            || part.path.to_ascii_lowercase().contains("secret")
            || part.path.to_ascii_lowercase().contains("provider-config")
        {
            return Err(PersonalBackupError::ArchiveSecretIncluded);
        }
        if part.path.to_ascii_lowercase().contains(".sqlite") {
            return Err(PersonalBackupError::RawSqliteCopyForbidden);
        }
        let bytes = fs::read(&path).map_err(|_| PersonalBackupError::IncompleteBackup)?;
        let digest = format!("{:x}", Sha256::digest(&bytes));
        if digest != part.sha256 {
            return Err(PersonalBackupError::ArchiveTampered);
        }
    }
    Ok(manifest)
}

fn copy_verified_archive(
    archive_path: &Path,
    staging: &Path,
    manifest: &BackupArchiveManifest,
) -> Result<(), PersonalBackupError> {
    fs::create_dir_all(staging).map_err(archive_io)?;
    fs::copy(
        archive_path.join("manifest.json"),
        staging.join("manifest.json"),
    )
    .map_err(archive_io)?;
    for part in &manifest.parts {
        let from = archive_path.join(&part.path);
        let to = staging.join(&part.path);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(archive_io)?;
        }
        fs::copy(&from, &to).map_err(archive_io)?;
    }
    Ok(())
}

fn manifest_digest_of(archive_path: &Path) -> Result<String, PersonalBackupError> {
    let bytes = fs::read(archive_path.join("manifest.json")).map_err(archive_io)?;
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

fn validate_export_unit(unit: &BackupExportUnit) -> Result<(), PersonalBackupError> {
    if unit.resource_id.trim().is_empty() {
        return Err(PersonalBackupError::MissingBinding);
    }
    if unit.content_digest.trim().is_empty() {
        return Err(PersonalBackupError::MissingDigest);
    }
    if export_text_is_contaminated(&unit.resource_id)
        || export_text_is_contaminated(&unit.content_digest)
        || unit
            .related_digest
            .as_deref()
            .is_some_and(export_text_is_contaminated)
        || unit
            .binding_revision_id
            .as_deref()
            .is_some_and(export_text_is_contaminated)
    {
        return Err(PersonalBackupError::SecretContamination);
    }

    match unit.kind {
        BackupExportKind::Memory => {
            let Some(related) = unit.related_digest.as_deref() else {
                return Err(PersonalBackupError::MissingDigest);
            };
            if related.trim().is_empty() {
                return Err(PersonalBackupError::MissingDigest);
            }
        }
        BackupExportKind::SkillPackage | BackupExportKind::SkillRevision => {
            if unit
                .related_digest
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            {
                return Err(PersonalBackupError::MissingDigest);
            }
        }
        BackupExportKind::SkillBinding => {
            let Some(revision_id) = unit.binding_revision_id.as_deref() else {
                return Err(PersonalBackupError::MissingBinding);
            };
            if revision_id.trim().is_empty() {
                return Err(PersonalBackupError::MissingBinding);
            }
            let Some(manifest_digest) = unit.related_digest.as_deref() else {
                return Err(PersonalBackupError::MissingBinding);
            };
            if manifest_digest.trim().is_empty() {
                return Err(PersonalBackupError::MissingBinding);
            }
        }
    }
    Ok(())
}

fn bind_export_plan_digest(units: &[BackupExportUnit]) -> String {
    let mut hasher = Sha256::new();
    for unit in units {
        hasher.update(export_kind_label(unit.kind).as_bytes());
        hasher.update(b"\0");
        hasher.update(unit.resource_id.as_bytes());
        hasher.update(b"\0");
        hasher.update(unit.inventory_category.as_bytes());
        hasher.update(b"\0");
        hasher.update(unit.content_digest.as_bytes());
        hasher.update(b"\0");
        if let Some(related) = unit.related_digest.as_deref() {
            hasher.update(related.as_bytes());
        }
        hasher.update(b"\0");
        if let Some(revision_id) = unit.binding_revision_id.as_deref() {
            hasher.update(revision_id.as_bytes());
        }
        hasher.update(b"\n");
    }
    format!("{:x}", hasher.finalize())
}

fn export_kind_label(kind: BackupExportKind) -> &'static str {
    match kind {
        BackupExportKind::Memory => "memory",
        BackupExportKind::SkillPackage => "skill-package",
        BackupExportKind::SkillRevision => "skill-revision",
        BackupExportKind::SkillBinding => "skill-binding",
    }
}

fn export_text_is_contaminated(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    FORBIDDEN_BACKUP_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
}

fn inventory_path_is_forbidden(path: &Path, excluded_secret_paths: &[PathBuf]) -> bool {
    if excluded_secret_paths
        .iter()
        .any(|secret| path == secret.as_path())
    {
        return true;
    }
    let lowered = path.to_string_lossy().to_ascii_lowercase();
    FORBIDDEN_BACKUP_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_layout() -> PersonalDataLayout {
        PersonalDataLayout::from_xdg_roots(
            PathBuf::from("/tmp/cos-config"),
            PathBuf::from("/tmp/cos-data"),
            PathBuf::from("/tmp/cos-state"),
            PathBuf::from("/tmp/cos-cache"),
            PathBuf::from("/tmp/cos-runtime"),
        )
    }

    fn sample_units() -> Vec<BackupExportUnit> {
        vec![
            BackupExportUnit {
                kind: BackupExportKind::Memory,
                resource_id: "mem-1".to_owned(),
                inventory_category: "authority-db",
                content_digest: "aa".repeat(32),
                related_digest: Some("bb".repeat(32)),
                binding_revision_id: None,
            },
            BackupExportUnit {
                kind: BackupExportKind::SkillPackage,
                resource_id: "pkg-1".to_owned(),
                inventory_category: "authority-db",
                content_digest: "cc".repeat(32),
                related_digest: None,
                binding_revision_id: None,
            },
            BackupExportUnit {
                kind: BackupExportKind::SkillRevision,
                resource_id: "rev-1".to_owned(),
                inventory_category: "authority-db",
                content_digest: "dd".repeat(32),
                related_digest: Some("cc".repeat(32)),
                binding_revision_id: None,
            },
            BackupExportUnit {
                kind: BackupExportKind::SkillBinding,
                resource_id: "bind-1".to_owned(),
                inventory_category: "authority-db",
                content_digest: "dd".repeat(32),
                related_digest: Some("cc".repeat(32)),
                binding_revision_id: Some("rev-1".to_owned()),
            },
        ]
    }

    #[test]
    fn plans_inventory_without_secret_paths() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).expect("plan must succeed");
        assert_eq!(inventory.entries.len(), 6);
        assert!(
            inventory
                .excluded_secret_paths
                .iter()
                .any(|path| path.ends_with("local-bootstrap.secret"))
        );
        assert!(
            inventory
                .excluded_secret_paths
                .iter()
                .any(|path| path.ends_with("provider-config.json"))
        );
        assert!(
            !inventory
                .entries
                .iter()
                .any(|entry| entry.path.ends_with("local-bootstrap.secret"))
        );
    }

    #[test]
    fn rejects_secret_path_reintroduction() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let mut poisoned = inventory.entries.clone();
        poisoned.push(BackupInventoryEntry {
            category: "secret-leak",
            path: layout.local_bootstrap_secret_path(),
        });
        assert_eq!(
            validate_backup_inventory(&poisoned, &inventory.excluded_secret_paths).unwrap_err(),
            PersonalBackupError::SecretPathIncluded
        );
    }

    #[test]
    fn rejects_missing_required_category() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let truncated: Vec<_> = inventory
            .entries
            .into_iter()
            .filter(|entry| entry.category != "artifacts")
            .collect();
        assert_eq!(
            validate_backup_inventory(&truncated, &[]).unwrap_err(),
            PersonalBackupError::MissingRequiredCategory("artifacts")
        );
    }

    #[test]
    fn plans_digest_bound_memory_skill_export() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let units = sample_units();
        let plan = plan_memory_skill_export(&inventory, &units).expect("export must succeed");
        assert_eq!(plan.units.len(), 4);
        assert!(plan.inventory_categories.contains(&"authority-db"));
        assert_eq!(plan.plan_digest.len(), 64);
        assert!(
            plan.units
                .iter()
                .all(|unit| unit.inventory_category == "authority-db")
        );
    }

    #[test]
    fn rejects_secret_contamination_in_export_unit() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let mut units = sample_units();
        units[0].resource_id = "mem-with-api_key-leak".to_owned();
        assert_eq!(
            plan_memory_skill_export(&inventory, &units).unwrap_err(),
            PersonalBackupError::SecretContamination
        );
    }

    #[test]
    fn rejects_missing_skill_binding_material() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let mut units = sample_units();
        units[3].binding_revision_id = None;
        assert_eq!(
            plan_memory_skill_export(&inventory, &units).unwrap_err(),
            PersonalBackupError::MissingBinding
        );

        units = sample_units();
        units[3].related_digest = None;
        assert_eq!(
            plan_memory_skill_export(&inventory, &units).unwrap_err(),
            PersonalBackupError::MissingBinding
        );
    }

    #[test]
    fn rejects_unapproved_export_category() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let mut units = sample_units();
        units[0].inventory_category = "config";
        assert_eq!(
            plan_memory_skill_export(&inventory, &units).unwrap_err(),
            PersonalBackupError::UnapprovedInventoryCategory
        );
    }

    fn sample_restore_candidate(inventory: &PersonalBackupInventory) -> BackupRestoreCandidate {
        let plan = plan_memory_skill_export(inventory, &sample_units()).unwrap();
        BackupRestoreCandidate {
            export_plan: plan,
            categories_present: inventory
                .entries
                .iter()
                .map(|entry| entry.category)
                .collect(),
            backup_schema_version: 23,
            expected_schema_version: 23,
            migration_plan_digest: "ee".repeat(32),
        }
    }

    #[test]
    fn accepts_compatible_complete_restore_preflight() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let candidate = sample_restore_candidate(&inventory);
        let preflight =
            preflight_personal_backup_restore(&inventory, &candidate).expect("preflight");
        assert_eq!(
            preflight.export_plan_digest,
            candidate.export_plan.plan_digest
        );
        assert_eq!(preflight.backup_schema_version, 23);
    }

    #[test]
    fn rejects_incompatible_schema_versions() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let mut candidate = sample_restore_candidate(&inventory);
        candidate.backup_schema_version = 22;
        assert_eq!(
            preflight_personal_backup_restore(&inventory, &candidate).unwrap_err(),
            PersonalBackupError::SchemaIncompatible
        );
    }

    #[test]
    fn rejects_incomplete_backup_categories() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let mut candidate = sample_restore_candidate(&inventory);
        candidate
            .categories_present
            .retain(|category| *category != "artifacts");
        assert_eq!(
            preflight_personal_backup_restore(&inventory, &candidate).unwrap_err(),
            PersonalBackupError::IncompleteBackup
        );
    }

    #[test]
    fn rejects_migration_digest_and_plan_digest_mismatches() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let mut candidate = sample_restore_candidate(&inventory);
        candidate.migration_plan_digest.clear();
        assert_eq!(
            preflight_personal_backup_restore(&inventory, &candidate).unwrap_err(),
            PersonalBackupError::MigrationPreflightFailed
        );

        candidate = sample_restore_candidate(&inventory);
        candidate.export_plan.plan_digest = "00".repeat(32);
        assert_eq!(
            preflight_personal_backup_restore(&inventory, &candidate).unwrap_err(),
            PersonalBackupError::ExportPlanDigestMismatch
        );
    }

    #[test]
    fn plans_and_commits_transactional_update_over_preflight() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let candidate = sample_restore_candidate(&inventory);
        let preflight = preflight_personal_backup_restore(&inventory, &candidate).unwrap();
        let plan =
            plan_personal_lifecycle(&preflight, PersonalLifecycleOperation::Update, &[], false)
                .expect("update plan");
        assert!(plan.staged);
        let committed = commit_personal_lifecycle(&plan).expect("commit");
        assert!(!committed.staged);
        assert_eq!(committed.operation, PersonalLifecycleOperation::Update);
    }

    #[test]
    fn rollback_aborts_without_commit() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let candidate = sample_restore_candidate(&inventory);
        let preflight = preflight_personal_backup_restore(&inventory, &candidate).unwrap();
        let plan =
            plan_personal_lifecycle(&preflight, PersonalLifecycleOperation::Rollback, &[], false)
                .unwrap();
        let aborted = abort_personal_lifecycle(&plan).unwrap();
        assert!(!aborted.staged);
        assert!(aborted.uninstall_targets.is_empty());
    }

    #[test]
    fn uninstall_refuses_secret_and_unconfirmed_data() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let candidate = sample_restore_candidate(&inventory);
        let preflight = preflight_personal_backup_restore(&inventory, &candidate).unwrap();
        assert_eq!(
            plan_personal_lifecycle(
                &preflight,
                PersonalLifecycleOperation::Uninstall,
                &[UninstallTargetClass::Secret],
                true,
            )
            .unwrap_err(),
            PersonalBackupError::UninstallConfirmationRequired
        );
        assert_eq!(
            plan_personal_lifecycle(
                &preflight,
                PersonalLifecycleOperation::Uninstall,
                &[UninstallTargetClass::Data],
                false,
            )
            .unwrap_err(),
            PersonalBackupError::UninstallConfirmationRequired
        );
        let plan = plan_personal_lifecycle(
            &preflight,
            PersonalLifecycleOperation::Uninstall,
            &[UninstallTargetClass::Config, UninstallTargetClass::Data],
            true,
        )
        .expect("confirmed data uninstall");
        assert_eq!(
            plan.uninstall_targets,
            vec![UninstallTargetClass::Config, UninstallTargetClass::Data]
        );
    }

    #[test]
    fn commit_refuses_unstaged_lifecycle_plan() {
        let layout = sample_layout();
        let inventory = plan_personal_backup_inventory(&layout, &[]).unwrap();
        let candidate = sample_restore_candidate(&inventory);
        let preflight = preflight_personal_backup_restore(&inventory, &candidate).unwrap();
        let plan =
            plan_personal_lifecycle(&preflight, PersonalLifecycleOperation::Update, &[], false)
                .unwrap();
        let committed = commit_personal_lifecycle(&plan).unwrap();
        assert_eq!(
            commit_personal_lifecycle(&committed).unwrap_err(),
            PersonalBackupError::LifecycleNotStaged
        );
    }

    fn hermetic_layout() -> (PersonalDataLayout, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "cos-p2t27-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let layout = PersonalDataLayout::from_xdg_roots(
            root.join("config"),
            root.join("data"),
            root.join("state"),
            root.join("cache"),
            root.join("runtime"),
        );
        layout.ensure_directories().expect("layout dirs");
        fs::write(layout.config_dir().join("ui.json"), b"{\"theme\":\"dark\"}").unwrap();
        fs::write(
            layout.config_dir().join("provider-config.json"),
            b"{\"secret_ref\":\"ssv1:should-not-copy\"}",
        )
        .unwrap();
        fs::write(layout.local_bootstrap_secret_path(), b"bootstrap-secret").unwrap();
        fs::write(layout.authority_database_path(), b"sqlite-bytes").unwrap();
        (layout, root)
    }

    #[test]
    fn archive_roundtrip_excludes_secrets_and_sqlite() {
        let (layout, root) = hermetic_layout();
        let receipt = write_personal_backup_archive(&layout).expect("write archive");
        assert!(!receipt.sqlite_copied);
        assert!(receipt.excluded_secret_count >= 2);
        let manifest = fs::read_to_string(receipt.archive_path.join("manifest.json")).unwrap();
        assert!(!manifest.contains("ssv1:"));
        assert!(!manifest.contains("bootstrap-secret"));
        assert!(
            !receipt
                .archive_path
                .join("parts/config/provider-config.json")
                .exists()
        );
        assert!(
            !fs::read_to_string(receipt.archive_path.join("manifest.json"))
                .unwrap()
                .contains("authority.sqlite")
        );
        let restored =
            restore_personal_backup_archive(&layout, &receipt.archive_path).expect("restore");
        assert!(restored.live_applied);
        assert!(restored.restored_path.join("manifest.json").exists());
        assert_eq!(
            fs::read(layout.config_dir().join("ui.json")).unwrap(),
            b"{\"theme\":\"dark\"}"
        );
        fs::write(layout.config_dir().join("ui.json"), b"mutated").unwrap();
        restore_personal_backup_archive(&layout, &receipt.archive_path).expect("restore again");
        assert_eq!(
            fs::read(layout.config_dir().join("ui.json")).unwrap(),
            b"{\"theme\":\"dark\"}"
        );
        assert_eq!(
            fs::read(layout.authority_database_path()).unwrap(),
            b"sqlite-bytes"
        );
        assert!(layout.config_dir().join("provider-config.json").exists());
        let archive_text = fs::read_to_string(receipt.archive_path.join("manifest.json")).unwrap();
        assert!(!archive_text.contains("ssv1:"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn restore_injected_fault_leaves_live_state() {
        let (layout, root) = hermetic_layout();
        let receipt = write_personal_backup_archive(&layout).unwrap();
        fs::write(layout.config_dir().join("ui.json"), b"live-only").unwrap();
        assert_eq!(
            restore_personal_backup_archive_with_options(
                &layout,
                &receipt.archive_path,
                BackupRestoreOptions {
                    apply_live: true,
                    inject_fault_before_apply: true,
                    refuse_if_daemon_lock: true,
                },
            )
            .unwrap_err(),
            PersonalBackupError::RestorePartialRefused
        );
        assert_eq!(
            fs::read(layout.config_dir().join("ui.json")).unwrap(),
            b"live-only"
        );
        assert_eq!(
            fs::read(layout.authority_database_path()).unwrap(),
            b"sqlite-bytes"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn restore_fresh_layout_excludes_secrets_and_sqlite() {
        let (source, source_root) = hermetic_layout();
        let receipt = write_personal_backup_archive(&source).unwrap();
        let dest_root = std::env::temp_dir().join(format!(
            "cos-p2t27-dest-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let dest = PersonalDataLayout::from_xdg_roots(
            dest_root.join("config"),
            dest_root.join("data"),
            dest_root.join("state"),
            dest_root.join("cache"),
            dest_root.join("runtime"),
        );
        dest.ensure_directories().unwrap();
        restore_personal_backup_archive(&dest, &receipt.archive_path).expect("restore dest");
        assert_eq!(
            fs::read(dest.config_dir().join("ui.json")).unwrap(),
            b"{\"theme\":\"dark\"}"
        );
        assert!(!dest.config_dir().join("provider-config.json").exists());
        assert!(!dest.authority_database_path().exists());
        assert!(!dest.local_bootstrap_secret_path().exists());
        let _ = fs::remove_dir_all(source_root);
        let _ = fs::remove_dir_all(dest_root);
    }

    #[test]
    fn restore_rejects_tampered_part() {
        let (layout, root) = hermetic_layout();
        let receipt = write_personal_backup_archive(&layout).unwrap();
        let export = receipt.archive_path.join("parts/authority-db/export.json");
        fs::write(&export, b"{\"tampered\":true}").unwrap();
        assert_eq!(
            restore_personal_backup_archive(&layout, &receipt.archive_path).unwrap_err(),
            PersonalBackupError::ArchiveTampered
        );
        assert_eq!(
            fs::read(layout.authority_database_path()).unwrap(),
            b"sqlite-bytes"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn restore_rejects_missing_category() {
        let (layout, root) = hermetic_layout();
        let receipt = write_personal_backup_archive(&layout).unwrap();
        fs::remove_dir_all(receipt.archive_path.join("parts/state")).unwrap();
        let error = restore_personal_backup_archive(&layout, &receipt.archive_path).unwrap_err();
        assert!(matches!(
            error,
            PersonalBackupError::IncompleteBackup | PersonalBackupError::ArchiveTampered
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn restore_rejects_incompatible_archive_schema() {
        let (layout, root) = hermetic_layout();
        let receipt = write_personal_backup_archive(&layout).unwrap();
        let manifest_path = receipt.archive_path.join("manifest.json");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
        manifest["backup_schema_version"] = serde_json::json!(99);
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert_eq!(
            restore_personal_backup_archive(&layout, &receipt.archive_path).unwrap_err(),
            PersonalBackupError::SchemaIncompatible
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn restore_refuses_when_external_daemon_lock_is_present() {
        let (layout, root) = hermetic_layout();
        let receipt = write_personal_backup_archive(&layout).unwrap();
        fs::write(layout.daemon_lock_path(), b"pid=1").unwrap();
        assert_eq!(
            restore_personal_backup_archive(&layout, &receipt.archive_path).unwrap_err(),
            PersonalBackupError::DaemonLockHeld
        );
        restore_personal_backup_archive_with_options(
            &layout,
            &receipt.archive_path,
            BackupRestoreOptions {
                apply_live: true,
                inject_fault_before_apply: false,
                refuse_if_daemon_lock: false,
            },
        )
        .expect("daemon-owned restore ignores its own lock");
        let _ = fs::remove_dir_all(root);
    }
}

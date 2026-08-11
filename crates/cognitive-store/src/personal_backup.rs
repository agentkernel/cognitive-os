//! Personal user backup inventory and Memory/Skill export planning (P7-T02).
//!
//! D01 plans which layout paths may enter a user-facing backup archive.
//! Memory, Skill, bindings, state, config, and artifact roots are eligible.
//! Secret Store material, bootstrap secrets, and provider opaque refs stay
//! excluded.
//!
//! D02 builds a digest-bound Memory/Skill/bindings export plan that may only
//! reference D01-approved inventory categories and never carries secret
//! material. This module does not write archives, restore data, or claim
//! Gate/release outcomes.

use crate::layout::PersonalDataLayout;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use thiserror::Error;

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
        if !approved
            .iter()
            .any(|category| *category == unit.inventory_category)
        {
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
}

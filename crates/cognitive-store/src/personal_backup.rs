//! Personal user backup inventory authority path (P7-T02/D01).
//!
//! Plans which layout paths may enter a user-facing backup archive. Memory,
//! Skill, bindings, state, config, and artifact roots are eligible. Secret
//! Store material, bootstrap secrets, and provider opaque refs stay excluded.
//! This module does not write archives, restore data, or claim Gate/release
//! outcomes.

use crate::layout::PersonalDataLayout;
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

/// Fail-closed errors for backup inventory planning.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PersonalBackupError {
    #[error("backup inventory proposal includes a forbidden secret path")]
    SecretPathIncluded,
    #[error("backup inventory proposal is missing a required category: {0}")]
    MissingRequiredCategory(&'static str),
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
}

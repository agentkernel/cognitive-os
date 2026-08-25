//! Personal XDG Base Directory layout (P1-T01).
//!
//! Resolves config/data/state/cache/runtime roots, creates them with
//! restrictive permissions, and exposes the two durable SQLite paths plus
//! backup/scratch locations. This module does not start a daemon, mint
//! credentials, or write authority state.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Product directory name under each XDG root.
pub const PERSONAL_PRODUCT_DIR_NAME: &str = "cognitiveos";

/// Durable authority database file name under the data root.
pub const AUTHORITY_DATABASE_FILE_NAME: &str = "authority.sqlite";

/// Durable installation database file name under the data root.
pub const INSTALLATION_DATABASE_FILE_NAME: &str = "installations.sqlite";

/// Fail-closed outcomes while resolving or creating the Personal layout.
#[derive(Debug, Error)]
pub enum PersonalLayoutError {
    /// A required environment root is missing or empty.
    #[error("personal layout path resolution failed: {detail}")]
    PathResolution { detail: String },
    /// Creating or securing a directory/file failed.
    #[error("personal layout filesystem operation failed during {operation}: {source}")]
    Filesystem {
        operation: &'static str,
        #[source]
        source: io::Error,
    },
    /// Another migration or daemon lifecycle owner holds the exclusive lock.
    #[error("personal layout is locked by another process: {detail}")]
    LayoutLocked { detail: String },
    /// A migration adapter failure while preparing Personal databases.
    #[error("personal database preparation failed: {detail}")]
    DatabasePreparation { detail: String },
}

/// Resolved Personal directory and database locations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonalDataLayout {
    config_dir: PathBuf,
    data_dir: PathBuf,
    state_dir: PathBuf,
    cache_dir: PathBuf,
    runtime_dir: PathBuf,
}

impl PersonalDataLayout {
    /// Resolve layout roots from XDG environment variables with documented
    /// fallbacks. `$HOME` (or `USERPROFILE` on Windows) is required when an
    /// XDG variable is unset. `XDG_RUNTIME_DIR` has no portable fallback and
    /// fails closed when missing so sockets/scratch never land in shared temp.
    pub fn resolve_from_env() -> Result<Self, PersonalLayoutError> {
        let home_directory = env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .ok_or_else(|| PersonalLayoutError::PathResolution {
                detail: "HOME or USERPROFILE is required when XDG roots are unset".to_owned(),
            })?;

        let config_root = env_path_or_fallback("XDG_CONFIG_HOME", home_directory.join(".config"))?;
        let data_root =
            env_path_or_fallback("XDG_DATA_HOME", home_directory.join(".local").join("share"))?;
        let state_root = env_path_or_fallback(
            "XDG_STATE_HOME",
            home_directory.join(".local").join("state"),
        )?;
        let cache_root = env_path_or_fallback("XDG_CACHE_HOME", home_directory.join(".cache"))?;
        let runtime_root = match env::var_os("XDG_RUNTIME_DIR") {
            Some(value) if !value.is_empty() => PathBuf::from(value),
            _ => {
                return Err(PersonalLayoutError::PathResolution {
                    detail: "XDG_RUNTIME_DIR is required and must be non-empty".to_owned(),
                });
            }
        };

        Ok(Self::from_xdg_roots(
            config_root,
            data_root,
            state_root,
            cache_root,
            runtime_root,
        ))
    }

    /// Build a layout under explicit XDG-style roots (used by tests and
    /// hermetic fixtures). Product subdirectories are appended.
    pub fn from_xdg_roots(
        config_root: impl Into<PathBuf>,
        data_root: impl Into<PathBuf>,
        state_root: impl Into<PathBuf>,
        cache_root: impl Into<PathBuf>,
        runtime_root: impl Into<PathBuf>,
    ) -> Self {
        Self {
            config_dir: config_root.into().join(PERSONAL_PRODUCT_DIR_NAME),
            data_dir: data_root.into().join(PERSONAL_PRODUCT_DIR_NAME),
            state_dir: state_root.into().join(PERSONAL_PRODUCT_DIR_NAME),
            cache_dir: cache_root.into().join(PERSONAL_PRODUCT_DIR_NAME),
            runtime_dir: runtime_root.into().join(PERSONAL_PRODUCT_DIR_NAME),
        }
    }

    /// Configuration directory (`$XDG_CONFIG_HOME/cognitiveos`).
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Durable data directory (`$XDG_DATA_HOME/cognitiveos`).
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// State directory for backups and non-durable operator state.
    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    /// Cache directory (may be cleared without authority loss).
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Runtime directory for sockets, scratch DBs, and exclusive locks.
    pub fn runtime_dir(&self) -> &Path {
        &self.runtime_dir
    }

    /// Durable authority SQLite path.
    pub fn authority_database_path(&self) -> PathBuf {
        self.data_dir.join(AUTHORITY_DATABASE_FILE_NAME)
    }

    /// Durable installation SQLite path.
    pub fn installation_database_path(&self) -> PathBuf {
        self.data_dir.join(INSTALLATION_DATABASE_FILE_NAME)
    }

    /// Pre-migration backup directory under state.
    pub fn backups_dir(&self) -> PathBuf {
        self.state_dir.join("backups")
    }

    /// Ephemeral migration scratch directory under runtime.
    pub fn migration_scratch_dir(&self) -> PathBuf {
        self.runtime_dir.join("migration")
    }

    /// Exclusive migration lock path (not the full daemon single-instance lock).
    pub fn migration_lock_path(&self) -> PathBuf {
        self.runtime_dir.join("migration.lock")
    }

    /// Exclusive Personal daemon single-instance lock (P1-T04).
    pub fn daemon_lock_path(&self) -> PathBuf {
        self.runtime_dir.join("daemon.lock")
    }

    /// Default Unix domain socket path for the Personal daemon (P1-T04 / ADR-0019).
    pub fn daemon_socket_path(&self) -> PathBuf {
        self.runtime_dir.join("daemon.sock")
    }

    /// Private bootstrap secret file for local session issuance (P1-T04 / ADR-0019).
    ///
    /// Mode `0600` on Unix. Never stored in SQLite authority tables or Secret Service.
    pub fn local_bootstrap_secret_path(&self) -> PathBuf {
        self.runtime_dir.join("local-bootstrap.secret")
    }

    /// Create layout directories with restrictive permissions (0700 on Unix).
    ///
    /// Does not create database files and does not migrate schemas.
    pub fn ensure_directories(&self) -> Result<(), PersonalLayoutError> {
        for directory in [
            self.config_dir.as_path(),
            self.data_dir.as_path(),
            self.state_dir.as_path(),
            self.cache_dir.as_path(),
            self.runtime_dir.as_path(),
            self.backups_dir().as_path(),
            self.migration_scratch_dir().as_path(),
        ] {
            create_private_directory(directory)?;
        }
        Ok(())
    }
}

fn env_path_or_fallback(
    variable_name: &str,
    fallback: PathBuf,
) -> Result<PathBuf, PersonalLayoutError> {
    match env::var_os(variable_name) {
        Some(value) if !value.is_empty() => Ok(PathBuf::from(value)),
        Some(_) => Err(PersonalLayoutError::PathResolution {
            detail: format!("{variable_name} is set but empty"),
        }),
        None => Ok(fallback),
    }
}

/// Create a directory and, on Unix, force mode `0700`.
pub(crate) fn create_private_directory(path: &Path) -> Result<(), PersonalLayoutError> {
    fs::create_dir_all(path).map_err(|source| PersonalLayoutError::Filesystem {
        operation: "create private directory",
        source,
    })?;
    apply_unix_mode(path, 0o700)?;
    Ok(())
}

/// Restrict an existing file to owner read/write (`0600`) on Unix.
pub(crate) fn restrict_private_file(path: &Path) -> Result<(), PersonalLayoutError> {
    apply_unix_mode(path, 0o600)
}

#[cfg(unix)]
fn apply_unix_mode(path: &Path, mode: u32) -> Result<(), PersonalLayoutError> {
    use std::os::unix::fs::PermissionsExt;
    let permissions = fs::Permissions::from_mode(mode);
    fs::set_permissions(path, permissions).map_err(|source| PersonalLayoutError::Filesystem {
        operation: "set private permissions",
        source,
    })
}

#[cfg(not(unix))]
fn apply_unix_mode(_path: &Path, _mode: u32) -> Result<(), PersonalLayoutError> {
    // Windows ACL hardening is out of P1-T01 scope; CI validates Unix modes.
    Ok(())
}

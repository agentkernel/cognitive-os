//! Single-instance daemon lifecycle lock (P1-T04 / ADR-0019).

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Failures acquiring or releasing the Personal daemon single-instance lock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonLifecycleError {
    AlreadyRunning { detail: String },
    Io { detail: &'static str },
}

impl std::fmt::Display for DaemonLifecycleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRunning { detail } => {
                write!(formatter, "personal daemon already running: {detail}")
            }
            Self::Io { detail } => write!(formatter, "daemon lifecycle I/O failure: {detail}"),
        }
    }
}

impl std::error::Error for DaemonLifecycleError {}

/// Exclusive create-new lock file under the Personal runtime directory.
///
/// Drop removes the lock file so a clean shutdown allows restart. Crash leaves
/// the file; operators may remove a stale lock after confirming no process holds it.
pub struct DaemonSingleInstanceLock {
    lock_path: PathBuf,
    _file: File,
}

impl DaemonSingleInstanceLock {
    /// Acquire exclusive daemon ownership at `lock_path`.
    pub fn acquire(lock_path: impl Into<PathBuf>) -> Result<Self, DaemonLifecycleError> {
        let lock_path = lock_path.into();
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent).map_err(|_| DaemonLifecycleError::Io {
                detail: "failed to create daemon lock parent directory",
            })?;
            #[cfg(unix)]
            restrict_private_directory(parent)?;
        }
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                let payload = format!("pid={} purpose=personal-daemon\n", std::process::id());
                file.write_all(payload.as_bytes())
                    .map_err(|_| DaemonLifecycleError::Io {
                        detail: "failed to write daemon lock payload",
                    })?;
                #[cfg(unix)]
                restrict_private_file(&lock_path)?;
                Ok(Self {
                    lock_path,
                    _file: file,
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(DaemonLifecycleError::AlreadyRunning {
                    detail: format!(
                        "daemon.lock already exists at {}; refuse second instance",
                        lock_path.display()
                    ),
                })
            }
            Err(_) => Err(DaemonLifecycleError::Io {
                detail: "failed to create daemon lock file",
            }),
        }
    }

    pub fn path(&self) -> &Path {
        &self.lock_path
    }
}

impl Drop for DaemonSingleInstanceLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

#[cfg(unix)]
fn restrict_private_directory(path: &Path) -> Result<(), DaemonLifecycleError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|_| {
        DaemonLifecycleError::Io {
            detail: "failed to set daemon lock directory mode",
        }
    })
}

#[cfg(unix)]
fn restrict_private_file(path: &Path) -> Result<(), DaemonLifecycleError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(|_| {
        DaemonLifecycleError::Io {
            detail: "failed to set daemon lock file mode",
        }
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn second_acquire_fails_until_first_drop() {
        let temp = std::env::temp_dir().join(format!("cos-daemon-lock-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let lock_path = temp.join("daemon.lock");
        let first = DaemonSingleInstanceLock::acquire(&lock_path).unwrap();
        let second = DaemonSingleInstanceLock::acquire(&lock_path).unwrap_err();
        assert!(matches!(second, DaemonLifecycleError::AlreadyRunning { .. }));
        drop(first);
        let third = DaemonSingleInstanceLock::acquire(&lock_path).unwrap();
        drop(third);
        let _ = fs::remove_dir_all(&temp);
    }
}
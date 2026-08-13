//! Small durable record store shared by restart-reconcilable native sinks.
//!
//! Records are key-bound JSON documents published through a same-directory
//! synced temporary file.  A stable OS lock file serializes both threads and
//! daemon restarts; record names are hashes, never raw idempotency keys.

use cap_std::fs::Dir;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use super::secure_fs::{
    AnchoredWorkspace, SecureEntry, create_new_regular_file, open_entry_at,
    open_or_create_regular_file, remove_regular_file, sync_directory,
};

const STATE_KEY_DIGEST_DOMAIN: &str = "native-tool-executor-state-key/0.1";
// State records contain only non-secret bindings, digests and receipts, never
// fetched or mutated content.
const MAXIMUM_STATE_RECORD_BYTES: u64 = 1024 * 1024;
static STATE_TEMPORARY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) struct DurableExecutorStateStore {
    directory: Dir,
    absolute_path: PathBuf,
}

impl DurableExecutorStateStore {
    pub(crate) fn open(path: &Path) -> io::Result<Self> {
        let absolute_path = absolute_lexical_path(path)?;
        let directory = AnchoredWorkspace::open_or_create(&absolute_path)?.root_dir()?;
        Ok(Self {
            directory,
            absolute_path,
        })
    }

    pub(crate) fn ensure_outside_workspace(&self, workspace_root: &Path) -> io::Result<()> {
        let workspace_root = absolute_lexical_path(workspace_root)?;
        if self.absolute_path.starts_with(&workspace_root) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "durable executor state must be outside the approved workspace root",
            ));
        }
        Ok(())
    }

    pub(crate) fn lock_key(&self, namespace: &str, key: &str) -> io::Result<StateGuard<'_>> {
        let mut lock_file = self.open_lock_file(namespace, key)?;
        lock_file.lock()?;
        lock_file.seek(SeekFrom::Start(0))?;
        Ok(StateGuard {
            directory: &self.directory,
            lock_file,
            record_name: record_name(namespace, key)?,
        })
    }

    pub(crate) fn try_lock_key(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<StateGuard<'_>, StateLockError> {
        let mut lock_file = self
            .open_lock_file(namespace, key)
            .map_err(StateLockError::Io)?;
        match lock_file.try_lock() {
            Ok(()) => {}
            Err(std::fs::TryLockError::WouldBlock) => return Err(StateLockError::WouldBlock),
            Err(std::fs::TryLockError::Error(error)) => return Err(StateLockError::Io(error)),
        }
        lock_file
            .seek(SeekFrom::Start(0))
            .map_err(StateLockError::Io)?;
        Ok(StateGuard {
            directory: &self.directory,
            lock_file,
            record_name: record_name(namespace, key).map_err(StateLockError::Io)?,
        })
    }

    fn open_lock_file(&self, namespace: &str, key: &str) -> io::Result<File> {
        let lock_name = lock_name(namespace, key)?;
        open_or_create_regular_file(&self.directory, &lock_name)
    }
}

pub(crate) enum StateLockError {
    WouldBlock,
    Io(io::Error),
}

pub(crate) struct StateGuard<'a> {
    directory: &'a Dir,
    lock_file: File,
    record_name: OsString,
}

impl StateGuard<'_> {
    pub(crate) fn read<T: DeserializeOwned>(&self) -> io::Result<Option<T>> {
        let mut file = match open_entry_at(self.directory, &self.record_name)? {
            SecureEntry::Absent => return Ok(None),
            SecureEntry::File(file) => file,
            SecureEntry::Directory(_) | SecureEntry::Rejected => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "durable executor state record is not a regular file",
                ));
            }
        };
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(MAXIMUM_STATE_RECORD_BYTES + 1)
            .read_to_end(&mut bytes)?;
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAXIMUM_STATE_RECORD_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "durable executor state record exceeds its fixed ceiling",
            ));
        }
        serde_json::from_slice(&bytes).map(Some).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("durable executor state record is invalid: {error}"),
            )
        })
    }

    pub(crate) fn write<T: Serialize>(&self, value: &T) -> io::Result<()> {
        let bytes = serde_json::to_vec(value).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("durable executor state serialization failed: {error}"),
            )
        })?;
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAXIMUM_STATE_RECORD_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "durable executor state record exceeds its fixed ceiling",
            ));
        }
        let temporary_name = temporary_record_name(&self.record_name);
        let mut temporary_file =
            create_new_regular_file(self.directory, OsStr::new(&temporary_name))?;
        let publication = (|| -> io::Result<()> {
            temporary_file.write_all(&bytes)?;
            temporary_file.flush()?;
            temporary_file.sync_all()?;
            self.directory.rename(
                OsStr::new(&temporary_name),
                self.directory,
                &self.record_name,
            )?;
            sync_directory(self.directory)
        })();
        if let Err(error) = publication {
            let cleanup = remove_regular_file(self.directory, OsStr::new(&temporary_name));
            return match cleanup {
                Ok(_) => Err(error),
                Err(cleanup_error) => Err(io::Error::other(format!(
                    "state publication failed ({error}); temporary cleanup also failed ({cleanup_error})"
                ))),
            };
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn remove_record(&self) -> io::Result<bool> {
        remove_regular_file(self.directory, &self.record_name)
    }
}

impl Drop for StateGuard<'_> {
    fn drop(&mut self) {
        let _ = self.lock_file.unlock();
    }
}

fn state_key_digest(namespace: &str, key: &str) -> io::Result<String> {
    let binding = format!("{namespace}\0{key}");
    cognitive_contracts::canonical::digest(binding.as_bytes(), STATE_KEY_DIGEST_DOMAIN)
        .map(|digest| digest.trim_start_matches("sha256:").to_owned())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))
}

fn record_name(namespace: &str, key: &str) -> io::Result<OsString> {
    Ok(OsString::from(format!(
        "record-{}.json",
        state_key_digest(namespace, key)?
    )))
}

fn lock_name(namespace: &str, key: &str) -> io::Result<OsString> {
    Ok(OsString::from(format!(
        "lock-{}.lck",
        state_key_digest(namespace, key)?
    )))
}

fn temporary_record_name(record_name: &OsStr) -> OsString {
    let sequence = STATE_TEMPORARY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    OsString::from(format!(
        ".{}.{}.{}.tmp",
        record_name.to_string_lossy(),
        std::process::id(),
        sequence
    ))
}

fn absolute_lexical_path(path: &Path) -> io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "durable executor state path contains parent traversal",
                ));
            }
        }
    }
    Ok(normalized)
}

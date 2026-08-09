//! Private filesystem content-addressed storage for governed artifact bytes.
//!
//! The caller supplies only opaque content. References are SHA-256 digests and
//! are never interpreted as filesystem paths, so an untrusted reference cannot
//! escape the daemon-owned CAS root.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

const SHA256_REFERENCE_PREFIX: &str = "sha256:";
const ARTIFACT_URI_REFERENCE_PREFIX: &str = "artifact://sha256/";

#[derive(Debug, Error)]
pub enum ArtifactStoreError {
    #[error("artifact reference is malformed: {0}")]
    MalformedReference(String),
    #[error("artifact digest mismatch: expected {expected}, computed {computed}")]
    DigestMismatch { expected: String, computed: String },
    #[error("artifact exceeds the configured maximum of {maximum_bytes} bytes")]
    TooLarge { maximum_bytes: usize },
    #[error("artifact storage I/O failed: {0}")]
    Io(#[from] io::Error),
    #[error("artifact metadata is invalid: {0}")]
    Metadata(String),
    #[error("artifact access is not authorized")]
    AccessDenied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub reference: String,
    pub byte_length: usize,
    pub content_type: String,
}

/// A daemon-owned immutable content-addressed artifact store.
#[derive(Debug, Clone)]
pub struct ArtifactStore {
    root_directory: PathBuf,
    maximum_bytes: usize,
}

impl ArtifactStore {
    pub fn open(
        root_directory: impl AsRef<Path>,
        maximum_bytes: usize,
    ) -> Result<Self, ArtifactStoreError> {
        fs::create_dir_all(root_directory.as_ref())?;
        Ok(Self {
            root_directory: root_directory.as_ref().to_path_buf(),
            maximum_bytes,
        })
    }

    pub fn put(&self, content: &[u8]) -> Result<String, ArtifactStoreError> {
        let reference = format!("{SHA256_REFERENCE_PREFIX}{:x}", Sha256::digest(content));
        self.put_with_metadata(&reference, content, "application/octet-stream")?;
        Ok(reference)
    }

    pub fn put_with_metadata(
        &self,
        expected_reference: &str,
        content: &[u8],
        content_type: &str,
    ) -> Result<ArtifactMetadata, ArtifactStoreError> {
        self.put_expected(expected_reference, content)?;
        let metadata = ArtifactMetadata {
            reference: expected_reference.to_owned(),
            byte_length: content.len(),
            content_type: content_type.to_owned(),
        };
        let metadata_json = serde_json::to_vec(&metadata)
            .map_err(|error| ArtifactStoreError::Metadata(error.to_string()))?;
        let digest = parse_reference(expected_reference)?;
        let metadata_staging_path = self
            .root_directory
            .join(format!(".{digest}.metadata.partial"));
        fs::write(&metadata_staging_path, metadata_json)?;
        fs::rename(metadata_staging_path, self.metadata_path(digest))?;
        Ok(metadata)
    }

    pub fn put_expected(
        &self,
        expected_reference: &str,
        content: &[u8],
    ) -> Result<(), ArtifactStoreError> {
        if content.len() > self.maximum_bytes {
            return Err(ArtifactStoreError::TooLarge {
                maximum_bytes: self.maximum_bytes,
            });
        }
        let expected_digest = parse_reference(expected_reference)?;
        let computed_reference = format!("{SHA256_REFERENCE_PREFIX}{:x}", Sha256::digest(content));
        if expected_reference != computed_reference {
            return Err(ArtifactStoreError::DigestMismatch {
                expected: expected_reference.to_owned(),
                computed: computed_reference,
            });
        }
        let artifact_path = self.root_directory.join(expected_digest);
        if artifact_path.exists() {
            return Ok(());
        }
        let staging_path = self
            .root_directory
            .join(format!(".{expected_digest}.partial"));
        fs::write(&staging_path, content)?;
        fs::rename(staging_path, artifact_path)?;
        Ok(())
    }

    pub fn get(&self, reference: &str) -> Result<Option<Vec<u8>>, ArtifactStoreError> {
        self.get_authorized(reference, true)
    }

    /// Reads bytes only after the daemon's caller-specific policy check has
    /// admitted the request. The store never infers authorization from a
    /// digest or from filesystem reachability.
    pub fn get_authorized(
        &self,
        reference: &str,
        access_allowed: bool,
    ) -> Result<Option<Vec<u8>>, ArtifactStoreError> {
        if !access_allowed {
            return Err(ArtifactStoreError::AccessDenied);
        }
        let digest = parse_reference(reference)?;
        let artifact_path = self.root_directory.join(digest);
        match fs::read(artifact_path) {
            Ok(content) => {
                let computed_reference =
                    format!("{SHA256_REFERENCE_PREFIX}{:x}", Sha256::digest(&content));
                if computed_reference != reference {
                    return Err(ArtifactStoreError::DigestMismatch {
                        expected: reference.to_owned(),
                        computed: computed_reference,
                    });
                }
                Ok(Some(content))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn metadata(
        &self,
        reference: &str,
    ) -> Result<Option<ArtifactMetadata>, ArtifactStoreError> {
        let digest = parse_reference(reference)?;
        match fs::read(self.metadata_path(digest)) {
            Ok(metadata_json) => {
                let metadata: ArtifactMetadata = serde_json::from_slice(&metadata_json)
                    .map_err(|error| ArtifactStoreError::Metadata(error.to_string()))?;
                if metadata.reference != reference {
                    return Err(ArtifactStoreError::Metadata(
                        "metadata reference does not match requested artifact".to_owned(),
                    ));
                }
                Ok(Some(metadata))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    /// Resolves the durable Artifact URI form used by verification records.
    /// A reference is usable only when its published bytes remain present and
    /// still hash to the digest embedded in that URI.
    pub fn contains_artifact_uri(&self, artifact_uri: &str) -> Result<bool, ArtifactStoreError> {
        let digest = artifact_uri
            .strip_prefix(ARTIFACT_URI_REFERENCE_PREFIX)
            .ok_or_else(|| ArtifactStoreError::MalformedReference(artifact_uri.to_owned()))?;
        let storage_reference = format!("{SHA256_REFERENCE_PREFIX}{digest}");
        Ok(self.get(&storage_reference)?.is_some())
    }

    fn metadata_path(&self, digest: &str) -> PathBuf {
        self.root_directory.join(format!("{digest}.metadata.json"))
    }

    /// Removes only abandoned staging files from interrupted writes. Published
    /// CAS bytes and their metadata are never garbage-collected by this method.
    pub fn remove_incomplete_writes(&self) -> Result<usize, ArtifactStoreError> {
        let mut removed_file_count = 0;
        for directory_entry in fs::read_dir(&self.root_directory)? {
            let directory_entry = directory_entry?;
            let file_name = directory_entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };
            let is_staging_file = file_name.starts_with('.') && file_name.ends_with(".partial");
            if is_staging_file && directory_entry.file_type()?.is_file() {
                fs::remove_file(directory_entry.path())?;
                removed_file_count += 1;
            }
        }
        Ok(removed_file_count)
    }
}

fn parse_reference(reference: &str) -> Result<&str, ArtifactStoreError> {
    let Some(digest) = reference.strip_prefix(SHA256_REFERENCE_PREFIX) else {
        return Err(ArtifactStoreError::MalformedReference(reference.to_owned()));
    };
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ArtifactStoreError::MalformedReference(reference.to_owned()));
    }
    Ok(digest)
}

//! P3-T03 Artifact CAS failure-closed storage regressions.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use cognitive_store::{ArtifactStore, ArtifactStoreError};
use std::fs;

#[test]
fn rejects_digest_mismatch_without_publishing_bytes() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let store = ArtifactStore::open(temporary_directory.path(), 1024).unwrap();

    let result = store.put_expected(
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        b"trusted content",
    );

    assert!(matches!(
        result,
        Err(ArtifactStoreError::DigestMismatch { .. })
    ));
    assert!(
        store
            .get("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap()
            .is_none()
    );
}

#[test]
fn writes_immutable_content_and_refuses_path_traversal_fetches() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let store = ArtifactStore::open(temporary_directory.path(), 1024).unwrap();
    let reference = store.put(b"trusted content").unwrap();

    assert_eq!(
        store.get(&reference).unwrap(),
        Some(b"trusted content".to_vec())
    );
    assert_eq!(store.metadata(&reference).unwrap().unwrap().byte_length, 15);
    assert!(matches!(
        store.get("../authority.sqlite"),
        Err(ArtifactStoreError::MalformedReference(_))
    ));
    assert!(matches!(
        store.get_authorized(&reference, false),
        Err(ArtifactStoreError::AccessDenied)
    ));
}

#[test]
fn removes_abandoned_partial_writes_without_touching_published_artifacts() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let store = ArtifactStore::open(temporary_directory.path(), 1024).unwrap();
    let reference = store.put(b"published artifact").unwrap();
    let partial_path = temporary_directory
        .path()
        .join(".interrupted-write.partial");
    fs::write(&partial_path, b"incomplete bytes").unwrap();

    assert_eq!(store.remove_incomplete_writes().unwrap(), 1);
    assert!(!partial_path.exists());
    assert_eq!(
        store.get(&reference).unwrap(),
        Some(b"published artifact".to_vec())
    );
}

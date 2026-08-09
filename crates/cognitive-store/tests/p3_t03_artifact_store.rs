//! P3-T03 Artifact CAS failure-closed storage regressions.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use cognitive_store::{ArtifactStore, ArtifactStoreError};

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
}

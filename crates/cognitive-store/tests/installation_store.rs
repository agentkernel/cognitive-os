//! Durable installation-store acceptance tests (D-020).
//!
//! These tests deliberately exercise a second SQLite handle and a reopen: an
//! in-memory ledger or a transaction that exposes staging rows cannot satisfy
//! the authority boundary required before a managed AgentInstallation commit.

use cognitive_store::{
    InstallationCommit, InstallationEvidence, InstallationStoreError, SqliteInstallationStore,
};

fn pi_installation() -> Result<InstallationCommit, Box<dyn std::error::Error>> {
    Ok(InstallationCommit::new(
        "pkg://pi/0.81.1",
        "sha256:package-bytes",
        "sha256:adapter-policy",
        "sha256:sandbox-policy",
        "sha256:compatibility-report",
    )?)
}

fn custom_pi_installation() -> Result<InstallationCommit, Box<dyn std::error::Error>> {
    Ok(InstallationCommit::new_with_evidence(
        "pkg://pi/0.81.1-custom",
        "sha256:package-bytes",
        "sha256:adapter-policy",
        "sha256:sandbox-policy",
        "sha256:compatibility-report",
        InstallationEvidence::custom_user_provided(
            "principal://tenant-a/verified-operator",
            "file://tenant-a/pi-0.81.1.bundle",
            "sha256:lockfile",
            "custom_acknowledgement_bound",
        )?,
    )?)
}

#[test]
fn custom_acknowledgement_evidence_is_atomically_committed_and_survives_reopen()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("installation-authority.db");
    let writer = SqliteInstallationStore::open(&path)?;
    let reader = SqliteInstallationStore::open(&path)?;
    let commit = custom_pi_installation()?;

    writer.stage(&commit)?;
    assert!(reader.committed(commit.package_ref())?.is_none());

    writer.commit(commit.package_ref())?;
    drop(writer);
    drop(reader);

    let reopened = SqliteInstallationStore::open(&path)?;
    let recovered = reopened
        .committed(commit.package_ref())?
        .ok_or("committed Custom installation missing after reopen")?;
    assert_eq!(recovered, commit);
    assert_eq!(
        recovered
            .evidence()
            .ok_or("missing Custom confirmation evidence")?
            .source_mode(),
        "custom_user_provided"
    );
    Ok(())
}

#[test]
fn official_acquisition_lock_evidence_is_immutable_and_survives_reopen()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("installation-authority.db");
    let store = SqliteInstallationStore::open(&path)?;
    let acquisition_lock = r#"{"package":"@earendil-works/pi-coding-agent","version":"0.81.1","signed_lock_ref":"attestation://pi/lock-01"}"#;
    let commit = InstallationCommit::new_with_evidence(
        "pkg://@earendil-works/pi-coding-agent@0.81.1",
        "sha256:package-bytes",
        "sha256:adapter-policy",
        "sha256:sandbox-policy",
        "sha256:compatibility-report",
        InstallationEvidence::official_pi(acquisition_lock, "sha256:dependency-lock")?,
    )?;

    store.stage(&commit)?;
    store.commit(commit.package_ref())?;
    drop(store);

    let reopened = SqliteInstallationStore::open(&path)?;
    let recovered = reopened
        .committed(commit.package_ref())?
        .ok_or("missing official commit")?;
    assert_eq!(recovered, commit);
    assert_eq!(
        recovered
            .evidence()
            .and_then(InstallationEvidence::acquisition_lock),
        Some(acquisition_lock)
    );
    Ok(())
}

#[test]
fn commit_is_atomically_visible_to_a_second_store_handle() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("installation-authority.db");
    let writer = SqliteInstallationStore::open(&path)?;
    let reader = SqliteInstallationStore::open(&path)?;
    let commit = pi_installation()?;

    writer.stage(&commit)?;
    assert!(reader.committed(commit.package_ref())?.is_none());

    writer.commit(commit.package_ref())?;
    assert_eq!(reader.committed(commit.package_ref())?, Some(commit));
    Ok(())
}

#[test]
fn reopening_discards_uncommitted_staging_rows() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("installation-authority.db");
    let commit = pi_installation()?;

    {
        let store = SqliteInstallationStore::open(&path)?;
        store.stage(&commit)?;
        assert_eq!(store.staging_count()?, 1);
    }

    let reopened = SqliteInstallationStore::open(&path)?;
    reopened.recover_interrupted_staging()?;
    assert_eq!(reopened.staging_count()?, 0);
    assert!(reopened.committed(commit.package_ref())?.is_none());
    Ok(())
}

#[test]
fn opening_a_reader_does_not_discard_a_live_writer_staging_row()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("installation-authority.db");
    let writer = SqliteInstallationStore::open(&path)?;
    let commit = pi_installation()?;

    writer.stage(&commit)?;
    let reader = SqliteInstallationStore::open(&path)?;

    assert_eq!(reader.staging_count()?, 1);
    assert!(reader.committed(commit.package_ref())?.is_none());
    writer.commit(commit.package_ref())?;
    assert_eq!(reader.committed(commit.package_ref())?, Some(commit));
    Ok(())
}

#[test]
fn committed_installation_cannot_be_overwritten_by_a_later_stage()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("installation-authority.db");
    let store = SqliteInstallationStore::open(&path)?;
    let original = pi_installation()?;
    let replacement = InstallationCommit::new(
        original.package_ref(),
        "sha256:replacement-package",
        "sha256:replacement-adapter",
        "sha256:replacement-sandbox",
        "sha256:replacement-compatibility",
    )?;

    store.stage(&original)?;
    store.commit(original.package_ref())?;
    store.stage(&replacement)?;

    let error = match store.commit(replacement.package_ref()) {
        Ok(()) => return Err("expected immutable installation overwrite rejection".into()),
        Err(error) => error,
    };
    assert!(matches!(error, InstallationStoreError::Conflict { .. }));
    assert_eq!(store.committed(original.package_ref())?, Some(original));
    Ok(())
}

#[test]
fn root_activation_is_versioned_durable_and_compare_and_swap_fenced()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("installation-authority.db");
    let store = SqliteInstallationStore::open(&path)?;
    let first = store.activate_installation_root(
        "installation-root://personal/pi",
        None,
        "pkg://@earendil-works/pi-coding-agent@0.81.1",
        "official-lock-01",
    )?;
    assert_eq!(first.activation_version(), 1);

    let conflict = store.activate_installation_root(
        first.installation_root(),
        Some(0),
        first.package_ref(),
        first.acquisition_lock(),
    );
    assert!(matches!(
        conflict,
        Err(InstallationStoreError::Conflict { .. })
    ));
    assert_eq!(
        store.active_installation_root(first.installation_root())?,
        Some(first.clone())
    );
    drop(store);

    let reopened = SqliteInstallationStore::open(&path)?;
    assert_eq!(
        reopened.active_installation_root(first.installation_root())?,
        Some(first)
    );
    Ok(())
}

#[test]
fn quarantine_removes_only_fenced_pointer_and_preserves_immutable_installation_reference()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("installation-authority.db");
    let store = SqliteInstallationStore::open(&path)?;
    let acquisition_lock = "official-lock-01";
    let commit = InstallationCommit::new_with_evidence(
        "pkg://@earendil-works/pi-coding-agent@0.81.1",
        "sha256:package-bytes",
        "sha256:adapter-policy",
        "sha256:sandbox-policy",
        "sha256:compatibility-report",
        InstallationEvidence::official_pi(acquisition_lock, "sha256:dependency-lock")?,
    )?;
    store.stage(&commit)?;
    store.commit(commit.package_ref())?;
    let active = store.activate_installation_root(
        "installation-root://personal/pi",
        None,
        commit.package_ref(),
        acquisition_lock,
    )?;
    let unrelated = store.activate_installation_root(
        "installation-root://personal/unrelated",
        None,
        "pkg://unrelated",
        "lock-unrelated",
    )?;

    let quarantine = store.quarantine_active_installation_root(
        active.installation_root(),
        active.activation_version(),
        "stopped",
    )?;
    assert_eq!(quarantine.package_ref(), commit.package_ref());
    assert!(
        store
            .active_installation_root(active.installation_root())?
            .is_none()
    );
    assert_eq!(
        store.active_installation_root(unrelated.installation_root())?,
        Some(unrelated)
    );
    assert_eq!(store.committed(commit.package_ref())?, Some(commit));
    assert_eq!(
        store.installation_root_binding(active.installation_root(), active.activation_version())?,
        Some(active)
    );
    assert_eq!(
        store
            .installation_quarantine("installation-root://personal/pi", 1)?
            .as_ref()
            .map(|record| record.lifecycle_precondition()),
        Some("stopped")
    );
    Ok(())
}

#[test]
fn quarantine_rejects_unsafe_precondition_and_stale_fence_without_partial_write()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("installation-authority.db");
    let store = SqliteInstallationStore::open(&path)?;
    let active = store.activate_installation_root(
        "installation-root://personal/pi",
        None,
        "pkg://pi",
        "official-lock-01",
    )?;

    let active_error = store.quarantine_active_installation_root(
        active.installation_root(),
        active.activation_version(),
        "active",
    );
    assert!(matches!(
        active_error,
        Err(InstallationStoreError::InvalidCommit { .. })
    ));
    assert_eq!(
        store.active_installation_root(active.installation_root())?,
        Some(active.clone())
    );

    let fence_error = store.quarantine_active_installation_root(
        active.installation_root(),
        active.activation_version() + 1,
        "absent",
    );
    assert!(matches!(
        fence_error,
        Err(InstallationStoreError::Conflict { .. })
    ));
    assert_eq!(
        store.active_installation_root(active.installation_root())?,
        Some(active)
    );
    assert!(
        store
            .installation_quarantine("installation-root://personal/pi", 1)?
            .is_none()
    );
    Ok(())
}

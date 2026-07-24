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

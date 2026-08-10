#![allow(clippy::expect_used, clippy::unwrap_used)]

use cognitive_runtime::{
    AcceptingOfficialPiAcquisitionLockVerifier, AcceptingSignaturePort,
    DurableInstallationAuthority, OFFICIAL_NPM_ORIGIN, OFFICIAL_PI_PACKAGE, OFFICIAL_PI_VERSION,
    OfficialPiAcquisitionRequest, PackageInstallRequest, PiInstallationRootActivationRequest,
    acquire_official_pi_durable, activate_official_pi_root_durable, install_package_durable,
    package_artifact_digest, package_sha256_digest, package_sri_sha512,
    rollback_official_pi_root_durable,
};

fn official_request() -> OfficialPiAcquisitionRequest {
    let artifact = b"staged-official-pi-package".to_vec();
    let dependency_lock = b"locked-dependencies".to_vec();
    let artifact_digest = package_artifact_digest(&artifact).unwrap();
    let lock_digest = package_artifact_digest(&dependency_lock).unwrap();
    OfficialPiAcquisitionRequest {
        install: PackageInstallRequest {
            package_id: format!("pkg://{OFFICIAL_PI_PACKAGE}@{OFFICIAL_PI_VERSION}"),
            publisher: OFFICIAL_PI_PACKAGE.to_owned(),
            package_version: OFFICIAL_PI_VERSION.to_owned(),
            artifact: artifact.clone(),
            declared_artifact_digest: artifact_digest,
            signature_ref: "official-lock".to_owned(),
            provenance_ref: OFFICIAL_NPM_ORIGIN.to_owned(),
            adapter_digest: "sha256:adapter".to_owned(),
            sandbox_digest: "sha256:sandbox".to_owned(),
            compatibility_digest: "sha256:compatibility".to_owned(),
            lockfile_digest: lock_digest.clone(),
            expected_adapter_digest: "sha256:adapter".to_owned(),
            expected_sandbox_digest: "sha256:sandbox".to_owned(),
            expected_compatibility_digest: "sha256:compatibility".to_owned(),
        },
        registry_origin: OFFICIAL_NPM_ORIGIN.to_owned(),
        resolved_origin: OFFICIAL_NPM_ORIGIN.to_owned(),
        sri_sha512: package_sri_sha512(&artifact),
        declared_package_sha256: package_sha256_digest(&artifact),
        dependency_lock,
        declared_dependency_lock_digest: lock_digest,
        node_version: "22.19.0".to_owned(),
        signed_acquisition_lock_ref: "attestation://pi/lock-01".to_owned(),
    }
}

#[test]
fn official_pi_lock_commits_immutable_evidence_without_activation() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let request = official_request();

    acquire_official_pi_durable(
        &manager,
        &request,
        &AcceptingOfficialPiAcquisitionLockVerifier,
    )
    .unwrap();

    let evidence = manager
        .committed_installation(&request.install.package_id)
        .unwrap()
        .unwrap()
        .evidence()
        .unwrap()
        .clone();
    assert_eq!(evidence.source_mode(), "official_pi");
    assert!(evidence.acquisition_lock().unwrap().contains("sha512-"));
    assert_eq!(authority.capability_grants(), 0);
}

#[test]
fn redirected_origin_is_rejected_without_commit() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let mut request = official_request();
    request.resolved_origin = "https://mirror.invalid/".to_owned();

    let error = acquire_official_pi_durable(
        &manager,
        &request,
        &AcceptingOfficialPiAcquisitionLockVerifier,
    )
    .unwrap_err();

    assert_eq!(error.code, "AGENT_PACKAGE_VERIFICATION_FAILED");
    assert!(!authority.is_committed(&request.install.package_id).unwrap());
}

fn activation_request(
    package_ref: String,
    expected_activation_version: Option<u64>,
) -> PiInstallationRootActivationRequest {
    PiInstallationRootActivationRequest {
        installation_root: "installation-root://personal/pi".to_owned(),
        package_ref,
        expected_activation_version,
        compatibility_accepted: true,
        health_accepted: true,
    }
}

#[test]
fn activation_rejects_uncommitted_or_non_official_acquisition_without_pointer() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let request = official_request();

    let error = activate_official_pi_root_durable(
        &manager,
        &activation_request(request.install.package_id.clone(), None),
    )
    .unwrap_err();

    assert_eq!(error.code, "AGENT_PACKAGE_VERIFICATION_FAILED");
    assert!(
        manager
            .active_installation_root("installation-root://personal/pi")
            .unwrap()
            .is_none()
    );

    let mut non_official_install = request.install;
    non_official_install.package_id = "pkg://unverified-pi".to_owned();
    install_package_durable(&manager, &non_official_install, &AcceptingSignaturePort).unwrap();
    let error = activate_official_pi_root_durable(
        &manager,
        &activation_request(non_official_install.package_id, None),
    )
    .unwrap_err();
    assert_eq!(error.code, "AGENT_PACKAGE_VERIFICATION_FAILED");
    assert!(
        manager
            .active_installation_root("installation-root://personal/pi")
            .unwrap()
            .is_none()
    );
}

#[test]
fn compatibility_or_health_rejection_happens_before_active_pointer_publish() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let request = official_request();
    acquire_official_pi_durable(
        &manager,
        &request,
        &AcceptingOfficialPiAcquisitionLockVerifier,
    )
    .unwrap();
    let mut activation = activation_request(request.install.package_id, None);
    activation.compatibility_accepted = false;

    let error = activate_official_pi_root_durable(&manager, &activation).unwrap_err();

    assert_eq!(error.code, "AGENT_PACKAGE_VERIFICATION_FAILED");
    assert!(
        manager
            .active_installation_root(&activation.installation_root)
            .unwrap()
            .is_none()
    );

    activation.compatibility_accepted = true;
    activation.health_accepted = false;
    let error = activate_official_pi_root_durable(&manager, &activation).unwrap_err();
    assert_eq!(error.code, "AGENT_PACKAGE_VERIFICATION_FAILED");
    assert!(
        manager
            .active_installation_root(&activation.installation_root)
            .unwrap()
            .is_none()
    );
}

#[test]
fn failed_upgrade_preserves_complete_binding_and_competing_activation_conflicts() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let request = official_request();
    acquire_official_pi_durable(
        &manager,
        &request,
        &AcceptingOfficialPiAcquisitionLockVerifier,
    )
    .unwrap();
    let first = activate_official_pi_root_durable(
        &manager,
        &activation_request(request.install.package_id.clone(), None),
    )
    .unwrap();

    let mut failed_upgrade = activation_request(request.install.package_id.clone(), Some(1));
    failed_upgrade.health_accepted = false;
    assert!(activate_official_pi_root_durable(&manager, &failed_upgrade).is_err());
    assert_eq!(
        manager
            .active_installation_root(&failed_upgrade.installation_root)
            .unwrap(),
        Some(first.clone())
    );

    let conflict = activate_official_pi_root_durable(
        &manager,
        &activation_request(request.install.package_id, Some(0)),
    )
    .unwrap_err();
    assert_eq!(conflict.code, "STATE_CONFLICT");
    assert_eq!(
        manager
            .active_installation_root(&failed_upgrade.installation_root)
            .unwrap(),
        Some(first)
    );
}

#[test]
fn incomplete_rollback_target_has_no_success_receipt() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let request = official_request();
    acquire_official_pi_durable(
        &manager,
        &request,
        &AcceptingOfficialPiAcquisitionLockVerifier,
    )
    .unwrap();
    let active = activate_official_pi_root_durable(
        &manager,
        &activation_request(request.install.package_id, None),
    )
    .unwrap();

    let error = rollback_official_pi_root_durable(
        &manager,
        active.installation_root(),
        active.activation_version(),
        99,
    )
    .unwrap_err();

    assert_eq!(error.code, "AGENT_PACKAGE_VERIFICATION_FAILED");
    assert_eq!(
        manager
            .active_installation_root(active.installation_root())
            .unwrap(),
        Some(active)
    );
}

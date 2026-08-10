#![allow(clippy::expect_used, clippy::unwrap_used)]

use cognitive_runtime::{
    AcceptingOfficialPiAcquisitionLockVerifier, DurableInstallationAuthority, OFFICIAL_NPM_ORIGIN,
    OFFICIAL_PI_INSTALLATION_ROOT, OFFICIAL_PI_PACKAGE, OFFICIAL_PI_VERSION,
    OfficialPiAcquisitionRequest, OfficialPiAgentRegistrationRequest, PackageInstallRequest,
    PiInstallationRootActivationRequest, acquire_official_pi_durable,
    activate_official_pi_root_durable, package_artifact_digest, package_sha256_digest,
    package_sri_sha512, register_official_pi_agent_durable,
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

fn activate_official_root(
    manager: &cognitive_runtime::DurableInstallationManager<'_>,
) -> cognitive_store::InstallationRootBinding {
    let request = official_request();
    acquire_official_pi_durable(
        manager,
        &request,
        &AcceptingOfficialPiAcquisitionLockVerifier,
    )
    .unwrap();
    activate_official_pi_root_durable(
        manager,
        &PiInstallationRootActivationRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            package_ref: request.install.package_id.clone(),
            expected_activation_version: None,
            compatibility_accepted: true,
            health_accepted: true,
        },
    )
    .unwrap()
}

fn registration_request(expected_activation_version: u64) -> OfficialPiAgentRegistrationRequest {
    OfficialPiAgentRegistrationRequest {
        installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
        expected_activation_version,
        expected_adapter_digest: "sha256:adapter".to_owned(),
        protocol_digest: "sha256:sidecar-protocol".to_owned(),
        policy_digest: "sha256:personal-policy".to_owned(),
    }
}

#[test]
fn active_root_registers_inactive_instance_without_capability() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let binding = activate_official_root(&manager);

    let record = register_official_pi_agent_durable(
        &manager,
        &registration_request(binding.activation_version()),
    )
    .unwrap();

    assert_eq!(record.installation_root(), OFFICIAL_PI_INSTALLATION_ROOT);
    assert_eq!(record.lifecycle_state(), "registered");
    assert_eq!(record.fencing_epoch(), 1);
    assert_eq!(record.adapter_digest(), "sha256:adapter");
    assert_eq!(record.protocol_digest(), "sha256:sidecar-protocol");
    assert_eq!(authority.capability_grants(), 0);
    let current = manager
        .current_agent_registration(OFFICIAL_PI_INSTALLATION_ROOT)
        .unwrap()
        .unwrap();
    assert_eq!(current.registration_id(), record.registration_id());
    assert_eq!(current.instance_id(), record.instance_id());
}

#[test]
fn inactive_root_is_rejected_before_registration() {
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

    let error = register_official_pi_agent_durable(&manager, &registration_request(1)).unwrap_err();

    assert_eq!(error.code, "AGENT_PACKAGE_VERIFICATION_FAILED");
    assert!(
        manager
            .current_agent_registration(OFFICIAL_PI_INSTALLATION_ROOT)
            .unwrap()
            .is_none()
    );
}

#[test]
fn adapter_digest_mismatch_fails_closed() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let binding = activate_official_root(&manager);
    let mut request = registration_request(binding.activation_version());
    request.expected_adapter_digest = "sha256:other-adapter".to_owned();

    let error = register_official_pi_agent_durable(&manager, &request).unwrap_err();

    assert_eq!(error.code, "DIGEST_MISMATCH");
    assert!(
        manager
            .current_agent_registration(OFFICIAL_PI_INSTALLATION_ROOT)
            .unwrap()
            .is_none()
    );
}

#[test]
fn duplicate_registration_for_same_root_conflicts() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let binding = activate_official_root(&manager);
    let request = registration_request(binding.activation_version());
    register_official_pi_agent_durable(&manager, &request).unwrap();

    let error = register_official_pi_agent_durable(&manager, &request).unwrap_err();

    assert_eq!(error.code, "STATE_CONFLICT");
}

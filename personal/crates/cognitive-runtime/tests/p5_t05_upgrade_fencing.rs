#![allow(clippy::expect_used, clippy::unwrap_used)]

//! P5-T05/D02: upgrade/rollback/uninstall preserve process-bound fencing and
//! refuse pin/digest drift before SidecarSession activation.

use cognitive_runtime::{
    AcceptingOfficialPiAcquisitionLockVerifier, DurableInstallationAuthority, OFFICIAL_NPM_ORIGIN,
    OFFICIAL_PI_INSTALLATION_ROOT, OFFICIAL_PI_PACKAGE, OFFICIAL_PI_VERSION,
    OfficialPiAcquisitionRequest, OfficialPiAgentActivationRequest,
    OfficialPiAgentLifecycleRequest, OfficialPiAgentRegistrationRequest, PackageInstallRequest,
    PiInstallationLifecyclePrecondition, PiInstallationRootActivationRequest,
    PiInstallationUninstallRequest, acquire_official_pi_durable,
    activate_official_pi_agent_durable, activate_official_pi_root_durable,
    observe_official_pi_agent_health_durable, package_artifact_digest, package_sha256_digest,
    package_sri_sha512, register_official_pi_agent_durable, rollback_official_pi_root_durable,
    stop_official_pi_agent_durable, uninstall_official_pi_root_durable,
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

fn activate_registered(
    manager: &cognitive_runtime::DurableInstallationManager<'_>,
) -> (
    cognitive_store::InstallationRootBinding,
    cognitive_store::AgentRegistrationRecord,
    cognitive_store::SidecarSessionRecord,
) {
    let binding = activate_official_root(manager);
    let registered = register_official_pi_agent_durable(
        manager,
        &registration_request(binding.activation_version()),
    )
    .unwrap();
    let (activated, session) = activate_official_pi_agent_durable(
        manager,
        &OfficialPiAgentActivationRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: registered.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();
    (binding, activated, session)
}

#[test]
fn process_bound_blocks_upgrade_and_preserves_pointer() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let (binding, _activated, session) = activate_registered(&manager);
    assert!(session.process_bound());

    let error = activate_official_pi_root_durable(
        &manager,
        &PiInstallationRootActivationRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            package_ref: binding.package_ref().to_owned(),
            expected_activation_version: Some(binding.activation_version()),
            compatibility_accepted: true,
            health_accepted: true,
        },
    )
    .unwrap_err();
    assert_eq!(error.code, "STATE_CONFLICT");
    assert!(error.detail.contains("process-bound"));
    assert_eq!(
        manager
            .active_installation_root(OFFICIAL_PI_INSTALLATION_ROOT)
            .unwrap()
            .unwrap()
            .activation_version(),
        binding.activation_version()
    );
    let health =
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT).unwrap();
    assert!(health.process_bound());
    assert_eq!(authority.capability_grants(), 0);
}

#[test]
fn process_bound_blocks_uninstall_and_rollback() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let (binding, activated, session) = activate_registered(&manager);
    assert!(session.process_bound());

    let uninstall_error = uninstall_official_pi_root_durable(
        &manager,
        &PiInstallationUninstallRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_activation_version: binding.activation_version(),
            lifecycle_precondition: Some(PiInstallationLifecyclePrecondition::Stopped),
        },
    )
    .unwrap_err();
    assert_eq!(uninstall_error.code, "STATE_CONFLICT");
    assert!(
        manager
            .active_installation_root(OFFICIAL_PI_INSTALLATION_ROOT)
            .unwrap()
            .is_some()
    );

    let rollback_error = rollback_official_pi_root_durable(
        &manager,
        OFFICIAL_PI_INSTALLATION_ROOT,
        binding.activation_version(),
        binding.activation_version(),
    )
    .unwrap_err();
    assert_eq!(rollback_error.code, "STATE_CONFLICT");
    assert!(
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT)
            .unwrap()
            .process_bound()
    );
    assert_eq!(activated.lifecycle_state(), "active");
    assert_eq!(authority.capability_grants(), 0);
}

#[test]
fn stop_clears_binding_then_uninstall_quarantines() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let (binding, activated, _) = activate_registered(&manager);

    let stopped = stop_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: activated.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(stopped.lifecycle_state(), "stopped");
    assert!(
        !observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT)
            .unwrap()
            .process_bound()
    );

    let receipt = uninstall_official_pi_root_durable(
        &manager,
        &PiInstallationUninstallRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_activation_version: binding.activation_version(),
            lifecycle_precondition: Some(PiInstallationLifecyclePrecondition::Stopped),
        },
    )
    .unwrap();
    assert_eq!(receipt.activation_version, binding.activation_version());
    assert!(
        manager
            .active_installation_root(OFFICIAL_PI_INSTALLATION_ROOT)
            .unwrap()
            .is_none()
    );
    assert!(
        manager
            .installation_quarantine(OFFICIAL_PI_INSTALLATION_ROOT, binding.activation_version())
            .unwrap()
            .is_some()
    );
    assert_eq!(authority.capability_grants(), 0);
}

#[test]
fn root_upgrade_pin_drift_rejects_activation_before_session() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let binding = activate_official_root(&manager);
    let registered = register_official_pi_agent_durable(
        &manager,
        &registration_request(binding.activation_version()),
    )
    .unwrap();

    let upgraded = activate_official_pi_root_durable(
        &manager,
        &PiInstallationRootActivationRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            package_ref: binding.package_ref().to_owned(),
            expected_activation_version: Some(binding.activation_version()),
            compatibility_accepted: true,
            health_accepted: true,
        },
    )
    .unwrap();
    assert_eq!(
        upgraded.activation_version(),
        binding.activation_version() + 1
    );

    let error = activate_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentActivationRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: registered.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap_err();
    assert_eq!(error.code, "STATE_CONFLICT");
    assert!(error.detail.contains("pin/digest drift"));
    assert!(
        manager
            .current_sidecar_session(registered.instance_id())
            .unwrap()
            .is_none()
    );
    let health =
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT).unwrap();
    assert!(!health.process_bound());
    assert_eq!(health.lifecycle_state(), "registered");
    assert_eq!(authority.capability_grants(), 0);
}

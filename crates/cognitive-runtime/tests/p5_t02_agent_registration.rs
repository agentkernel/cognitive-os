#![allow(clippy::expect_used, clippy::unwrap_used)]

use cognitive_runtime::{
    AcceptingOfficialPiAcquisitionLockVerifier, DurableInstallationAuthority, OFFICIAL_NPM_ORIGIN,
    OFFICIAL_PI_INSTALLATION_ROOT, OFFICIAL_PI_PACKAGE, OFFICIAL_PI_VERSION,
    OfficialPiAcquisitionRequest, OfficialPiAgentActivationRequest,
    OfficialPiAgentLifecycleRequest, OfficialPiAgentRegistrationRequest, PackageInstallRequest,
    PiInstallationRootActivationRequest, acquire_official_pi_durable,
    activate_official_pi_agent_durable, activate_official_pi_root_durable,
    observe_official_pi_agent_health_durable, package_artifact_digest, package_sha256_digest,
    package_sri_sha512, pause_official_pi_agent_durable, recover_official_pi_agent_durable,
    register_official_pi_agent_durable, resume_official_pi_agent_durable,
    stop_official_pi_agent_durable,
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

#[test]
fn registered_instance_activates_sidecar_session_without_capability() {
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

    let (activated, session) = activate_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentActivationRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: registered.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();

    assert_eq!(activated.lifecycle_state(), "active");
    assert_eq!(activated.fencing_epoch(), 2);
    assert_eq!(session.lifecycle_state(), "active");
    assert_eq!(session.fencing_epoch(), 2);
    assert_eq!(session.protocol_digest(), "sha256:sidecar-protocol");
    assert_eq!(authority.capability_grants(), 0);
    let current = manager
        .current_sidecar_session(activated.instance_id())
        .unwrap()
        .unwrap();
    assert_eq!(current.session_id(), session.session_id());
}

#[test]
fn unregistered_root_cannot_activate() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    activate_official_root(&manager);

    let error = activate_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentActivationRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: 1,
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap_err();

    assert_eq!(error.code, "AGENT_PACKAGE_VERIFICATION_FAILED");
}

#[test]
fn protocol_digest_mismatch_rejects_activation() {
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

    let error = activate_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentActivationRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: registered.fencing_epoch(),
            protocol_digest: "sha256:other-protocol".to_owned(),
        },
    )
    .unwrap_err();

    assert_eq!(error.code, "PROTOCOL_SCHEMA_DIGEST_MISMATCH");
    assert!(
        manager
            .current_sidecar_session(registered.instance_id())
            .unwrap()
            .is_none()
    );
}

#[test]
fn duplicate_activation_conflicts() {
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
    let request = OfficialPiAgentActivationRequest {
        installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
        expected_fencing_epoch: registered.fencing_epoch(),
        protocol_digest: "sha256:sidecar-protocol".to_owned(),
    };
    activate_official_pi_agent_durable(&manager, &request).unwrap();

    let error = activate_official_pi_agent_durable(&manager, &request).unwrap_err();

    assert_eq!(error.code, "STATE_CONFLICT");
}

fn activate_registered(
    manager: &cognitive_runtime::DurableInstallationManager<'_>,
) -> (
    cognitive_store::AgentRegistrationRecord,
    cognitive_store::SidecarSessionRecord,
) {
    let binding = activate_official_root(manager);
    let registered = register_official_pi_agent_durable(
        manager,
        &registration_request(binding.activation_version()),
    )
    .unwrap();
    activate_official_pi_agent_durable(
        manager,
        &OfficialPiAgentActivationRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: registered.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap()
}

#[test]
fn active_instance_pause_resume_stop_fences_sessions() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let (activated, first_session) = activate_registered(&manager);

    let paused = pause_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: activated.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(paused.lifecycle_state(), "paused");
    assert_eq!(paused.fencing_epoch(), first_session.fencing_epoch());
    assert!(
        manager
            .current_sidecar_session(paused.instance_id())
            .unwrap()
            .is_none()
    );

    let (resumed, second_session) = resume_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: paused.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(resumed.lifecycle_state(), "active");
    assert_eq!(resumed.fencing_epoch(), first_session.fencing_epoch() + 1);
    assert_eq!(second_session.fencing_epoch(), resumed.fencing_epoch());
    assert_ne!(second_session.session_id(), first_session.session_id());
    assert_eq!(authority.capability_grants(), 0);

    let stopped = stop_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: resumed.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(stopped.lifecycle_state(), "stopped");
    assert!(
        manager
            .current_sidecar_session(stopped.instance_id())
            .unwrap()
            .is_none()
    );
}

#[test]
fn stale_pause_epoch_fails_closed() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let (activated, _) = activate_registered(&manager);

    let error = pause_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: activated.fencing_epoch() - 1,
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap_err();

    assert_eq!(error.code, "STATE_CONFLICT");
    assert!(
        manager
            .current_sidecar_session(activated.instance_id())
            .unwrap()
            .is_some()
    );
}

#[test]
fn health_and_recover_keep_identities_and_zero_capability() {
    let directory = tempfile::tempdir().unwrap();
    let authority =
        DurableInstallationAuthority::open(&directory.path().join("install.db")).unwrap();
    let manager = authority.acquire_installation_manager().unwrap();
    let (activated, session) = activate_registered(&manager);

    let health =
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT).unwrap();
    assert_eq!(health.instance_id(), activated.instance_id());
    assert_eq!(health.lifecycle_state(), "active");
    assert!(health.current_sidecar_session());
    assert_eq!(
        health.sidecar_fencing_epoch(),
        Some(session.fencing_epoch())
    );
    assert!(health.process_bound());
    assert_ne!(health.instance_id(), session.session_id());
    assert!(session.process_bound());
    assert!(session.process_attempt_id().is_some());
    assert_ne!(session.process_attempt_id().unwrap(), session.session_id());

    let stopped = stop_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: activated.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();
    let (recovered, recovered_session) = recover_official_pi_agent_durable(
        &manager,
        &OfficialPiAgentLifecycleRequest {
            installation_root: OFFICIAL_PI_INSTALLATION_ROOT.to_owned(),
            expected_fencing_epoch: stopped.fencing_epoch(),
            protocol_digest: "sha256:sidecar-protocol".to_owned(),
        },
    )
    .unwrap();
    assert_eq!(recovered.lifecycle_state(), "active");
    assert_eq!(recovered.fencing_epoch(), stopped.fencing_epoch() + 1);
    assert_ne!(recovered_session.session_id(), session.session_id());
    assert_eq!(authority.capability_grants(), 0);
    let recovered_health =
        observe_official_pi_agent_health_durable(&manager, OFFICIAL_PI_INSTALLATION_ROOT).unwrap();
    assert!(recovered_health.process_bound());
    assert!(recovered_session.process_bound());
    assert_eq!(
        recovered_health.sidecar_fencing_epoch(),
        Some(recovered_session.fencing_epoch())
    );
}
